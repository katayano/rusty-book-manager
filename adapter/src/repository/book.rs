//! BookのDB操作のための具象実装をするモジュール

use async_trait::async_trait;
use derive_new::new;
use kernel::model::{
    book::{
        Book, BookListOptions,
        event::{CreateBook, DeleteBook, UpdateBook},
    },
    id::{BookId, UserId},
    list::PaginatedList,
};
use kernel::repository::book::BookRepository;
use shared::error::AppError;

use crate::database::ConnectionPool;
use crate::database::model::book::{BookRow, PaginatedBookRow};
use shared::error::AppResult;

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

        // BookRowをBookに変換
        let items = rows.into_iter().map(Book::from).collect();
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
        Ok(row.map(Book::from))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::user::UserRepositoryImpl;
    use kernel::{model::user::event::CreateUser, repository::user::UserRepository};

    #[sqlx::test]
    async fn test_create_book(pool: sqlx::PgPool) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO roles(name) VALUES ('Admin'), ('User')
            "#,
        )
        .execute(&pool)
        .await?;

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
        } = book.unwrap();
        assert_eq!(id, book_id);
        assert_eq!(title, "Test Title");
        assert_eq!(author, "Test Author");
        assert_eq!(isbn, "Test ISBN");
        assert_eq!(description, "Test Description");
        assert_eq!(owner.name, user.name);
        Ok(())
    }
}
