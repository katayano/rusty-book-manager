//! BookのDB操作のための具象実装をするモジュール

use anyhow::Result;
use async_trait::async_trait;
use derive_new::new;
use kernel::model::book::{Book, event::CreateBook};
use kernel::repository::book::BookRepository;
use uuid::Uuid;

use crate::database::ConnectionPool;
use crate::database::model::book::BookRow;

#[derive(new)]
pub struct BookRepositoryImpl {
    db: ConnectionPool,
}

// NOTE: 「set DATABASE_URL to use query macros online～」の警告を消すためには
// cargo make sqlx-prepareを実行する必要がある
#[async_trait]
impl BookRepository for BookRepositoryImpl {
    /// 書籍を登録する
    async fn create(&self, event: CreateBook) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO books (title, author, isbn, description)
            VALUES ($1, $2, $3, $4)
            "#,
            event.title,
            event.author,
            event.isbn,
            event.description,
        )
        .execute(self.db.inner_ref())
        .await?;
        Ok(())
    }
    /// 書籍を全件取得する
    async fn find_all(&self) -> Result<Vec<Book>> {
        let rows = sqlx::query_as!(
            BookRow,
            r#"
            SELECT book_id, title, author, isbn, description
            FROM books
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(self.db.inner_ref())
        .await?;

        // BookRowをBookに変換して返す
        Ok(rows.into_iter().map(Book::from).collect())
    }
    /// 書籍を取得する
    async fn find_by_id(&self, book_id: Uuid) -> Result<Option<Book>> {
        let row = sqlx::query_as!(
            BookRow,
            r#"
            SELECT book_id, title, author, isbn, description
            FROM books
            WHERE book_id = $1
            "#,
            book_id,
        )
        .fetch_optional(self.db.inner_ref())
        .await?;

        // BookRowをBookに変換して返す
        Ok(row.map(Book::from))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn test_create_book(pool: sqlx::PgPool) -> anyhow::Result<()> {
        // BookRepositoryImplを初期化
        let repository = BookRepositoryImpl::new(ConnectionPool::new(pool));

        // CreateBookイベントを作成
        let book = CreateBook {
            title: "Test Title".into(),
            author: "Test Author".into(),
            isbn: "Test ISBN".into(),
            description: "Test Description".into(),
        };
        // 書籍を登録し、正常終了することを確認
        repository.create(book).await?;

        // 書籍を全件取得し、登録した書籍が含まれていることを確認
        let books = repository.find_all().await?;
        assert_eq!(books.len(), 1);

        // 取得した一覧から蔵書IDを取得し、
        // find_by_idで取得できることを確認
        let book_id = books[0].id;
        let book = repository.find_by_id(book_id).await?;
        assert!(book.is_some());

        // 取得した書籍の情報が登録した書籍と一致することを確認
        let Book {
            id,
            title,
            author,
            isbn,
            description,
        } = book.unwrap();
        assert_eq!(id, book_id);
        assert_eq!(title, "Test Title");
        assert_eq!(author, "Test Author");
        assert_eq!(isbn, "Test ISBN");
        assert_eq!(description, "Test Description");
        Ok(())
    }
}
