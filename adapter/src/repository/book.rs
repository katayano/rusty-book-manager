//! BookのDB操作のための具象実装をするモジュール

use async_trait::async_trait;
use derive_new::new;
use std::collections::HashMap;

use kernel::model::{
    book::{
        Book, BookListOptions, Checkout,
        event::{CreateBook, DeleteBook, UpdateBook},
    },
    id::{BookId, UserId},
    list::PaginatedList,
};
use kernel::repository::book::BookRepository;
use shared::error::{AppError, AppResult};

use crate::database::ConnectionPool;
use crate::database::model::book::{BookCheckoutRow, BookRow, PaginatedBookRow};

#[derive(new)]
pub struct BookRepositoryImpl {
    db: ConnectionPool,
}

// NOTE: 「set DATABASE_URL to use query macros online～」の警告を消すためには
// cargo make sqlx-prepareを実行する必要がある
#[async_trait]
impl BookRepository for BookRepositoryImpl {
    /// 書籍を登録する
    async fn create(&self, event: CreateBook, user_id: UserId) -> AppResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO books (title, author, isbn, description, user_id)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            event.title,
            event.author,
            event.isbn,
            event.description,
            user_id as _
        )
        .execute(self.db.inner_ref())
        .await
        // sqlx::Error型をAppError型に変換する
        .map_err(AppError::SpecificOperationError)?;

        Ok(())
    }
    /// 指定したlimit, offsetに応じて、書籍を取得する
    async fn find_all(&self, options: BookListOptions) -> AppResult<PaginatedList<Book>> {
        let BookListOptions { limit, offset } = options;

        let rows = sqlx::query_as!(
            PaginatedBookRow,
            r#"
            SELECT COUNT(*) OVER() AS "total!",
                b.book_id AS id
            FROM books as b
            ORDER BY b.created_at DESC
            LIMIT $1
            OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        // レコードがない場合はtotalを0にする
        let total = rows.first().map(|r| r.total).unwrap_or_default();
        let book_ids = rows.into_iter().map(|r| r.id).collect::<Vec<BookId>>();

        let rows: Vec<BookRow> = sqlx::query_as!(
            BookRow,
            r#"
            SELECT 
                b.book_id AS book_id, 
                b.title AS title, 
                b.author AS author, 
                b.isbn AS isbn, 
                b.description AS description,
                u.user_id AS owned_by,
                u.name AS owner_name
            FROM books AS b
            INNER JOIN users AS u USING(user_id)
            WHERE b.book_id IN (SELECT * FROM UNNEST($1::uuid[]))
            ORDER BY b.created_at DESC
            "#,
            &book_ids as _
        )
        .fetch_all(self.db.inner_ref())
        .await
        // sqlx::Error型をAppError型に変換する
        .map_err(AppError::SpecificOperationError)?;

        let book_ids = rows.iter().map(|book| book.book_id).collect::<Vec<_>>();
        let mut checkouts = self.find_checktouts(&book_ids).await?;

        let items = rows
            .into_iter()
            .map(|row| {
                let checkout = checkouts.remove(&row.book_id);
                row.into_book(checkout)
            })
            .collect();
        Ok(PaginatedList {
            total,
            limit,
            offset,
            items,
        })
    }
    /// 書籍を取得する
    async fn find_by_id(&self, book_id: BookId) -> AppResult<Option<Book>> {
        let row = sqlx::query_as!(
            BookRow,
            r#"
            SELECT 
                b.book_id as book_id, 
                b.title as title, 
                b.author as author, 
                b.isbn as isbn, 
                b.description as description,
                u.user_id as owned_by,
                u.name AS owner_name
            FROM books AS b
            INNER JOIN users AS u USING(user_id)
            WHERE b.book_id = $1
            "#,
            book_id as _, // query_as!マクロによるコンパイル時の型チェックを無効化（sqlx::query!マクロのドキュメントに記載されている）
        )
        .fetch_optional(self.db.inner_ref())
        .await
        // sqlx::Error型をAppError型に変換する
        .map_err(AppError::SpecificOperationError)?;

        // BookRowをBookに変換して返す
        match row {
            Some(row) => {
                let book_id = row.book_id;
                let mut checkouts = self.find_checktouts(&[book_id]).await?;
                let checkout = checkouts.remove(&book_id);
                Ok(Some(row.into_book(checkout)))
            }
            None => Ok(None),
        }
    }

    /// 書籍を更新する
    async fn update(&self, event: UpdateBook) -> AppResult<()> {
        let res = sqlx::query!(
            r#"
            UPDATE books
            SET title = $1, author = $2, isbn = $3, description = $4
            WHERE book_id = $5
            AND user_id = $6
            "#,
            event.title,
            event.author,
            event.isbn,
            event.description,
            event.book_id as _,
            // 内容を変更できるのは所有者のみ
            event.requested_user as _,
        )
        .execute(self.db.inner_ref())
        .await
        // sqlx::Error型をAppError型に変換する
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() == 0 {
            return Err(AppError::EntityNotFound(
                "specified book not found or you do not have permission to update it".into(),
            ));
        }

        Ok(())
    }

    /// 書籍を削除する
    async fn delete(&self, event: DeleteBook) -> AppResult<()> {
        let res = sqlx::query!(
            r#"
            DELETE FROM books
            WHERE book_id = $1
            AND user_id = $2
            "#,
            event.book_id as _,
            // 書籍を削除できるのは所有者のみ
            event.requested_user as _,
        )
        .execute(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() == 0 {
            return Err(AppError::EntityNotFound(
                "specified book not found or you do not have permission to delete it".into(),
            ));
        }

        Ok(())
    }
}

impl BookRepositoryImpl {
    /// 指定された書籍IDの貸出情報を取得する
    async fn find_checktouts(&self, book_ids: &[BookId]) -> AppResult<HashMap<BookId, Checkout>> {
        let res = sqlx::query_as!(
            BookCheckoutRow,
            r#"
            SELECT 
                c.checkout_id,
                c.book_id,
                u.user_id,
                u.name AS user_name,
                c.checked_out_at
            FROM checkouts AS c
            INNER JOIN users AS u USING(user_id)
            WHERE c.book_id = ANY($1)
            "#,
            &book_ids as _
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?
        .into_iter()
        .map(|checkout| (checkout.book_id, Checkout::from(checkout)))
        .collect();

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::repository::user::UserRepositoryImpl;
    use kernel::{model::user::event::CreateUser, repository::user::UserRepository};

    #[sqlx::test(fixtures("common"))]
    async fn test_create_book(pool: sqlx::PgPool) -> anyhow::Result<()> {
        // RepositoryImplを初期化
        let user_repository = UserRepositoryImpl::new(ConnectionPool::new(pool.clone()));
        let repository = BookRepositoryImpl::new(ConnectionPool::new(pool));

        // ユーザーを登録
        let user = user_repository
            .create(CreateUser {
                name: "Test User".into(),
                email: "test@example.com".into(),
                password: "test_password".into(),
            })
            .await?;

        // CreateBookイベントを作成
        let book = CreateBook {
            title: "Test Title".into(),
            author: "Test Author".into(),
            isbn: "Test ISBN".into(),
            description: "Test Description".into(),
        };
        // 書籍を登録し、正常終了することを確認
        repository.create(book, user.id).await?;

        // 書籍を取得し、登録した書籍が含まれていることを確認
        let options = BookListOptions {
            limit: 10,
            offset: 0,
        };
        let books = repository.find_all(options).await?;
        assert_eq!(books.items.len(), 1);

        // 取得した一覧から蔵書IDを取得し、
        // find_by_idで取得できることを確認
        let book_id = books.items[0].id;
        let book = repository.find_by_id(book_id).await?;
        assert!(book.is_some());

        // 取得した書籍の情報が登録した書籍と一致することを確認
        let Book {
            id,
            title,
            author,
            isbn,
            description,
            owner,
            ..
        } = book.unwrap();
        assert_eq!(id, book_id);
        assert_eq!(title, "Test Title");
        assert_eq!(author, "Test Author");
        assert_eq!(isbn, "Test ISBN");
        assert_eq!(description, "Test Description");
        assert_eq!(owner.name, user.name);
        Ok(())
    }

    #[sqlx::test(fixtures("common", "book"))]
    async fn test_update_book(pool: sqlx::PgPool) -> anyhow::Result<()> {
        // RepositoryImplを初期化
        let repository = BookRepositoryImpl::new(ConnectionPool::new(pool.clone()));

        // 更新する書籍のIDを指定
        let book_id = BookId::from_str("9890736e-a4e4-461a-a77d-eac3517ef11b").unwrap();
        let book = repository.find_by_id(book_id).await?.unwrap();
        const NEW_AUTHOR: &str = "New Author";
        assert_ne!(book.author, NEW_AUTHOR);

        // 書籍を更新
        let update_book = UpdateBook {
            book_id: book.id,
            title: book.title,
            author: NEW_AUTHOR.into(), // 更新箇所
            isbn: book.isbn,
            description: book.description,
            requested_user: UserId::from_str("5b4c96ac-316a-4bee-8e69-cac5eb84ff4c").unwrap(),
        };
        repository.update(update_book).await?;

        // 書籍を再度取得し、更新が反映されていることを確認
        let updated_book = repository.find_by_id(book_id).await?.unwrap();
        assert_eq!(updated_book.author, NEW_AUTHOR);
        Ok(())
    }
}
