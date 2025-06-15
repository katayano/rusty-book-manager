//! 蔵書のDB操作のための抽象実装をするモジュール
use async_trait::async_trait;

use crate::model::{
    book::{
        Book, BookListOptions,
        event::{CreateBook, DeleteBook, UpdateBook},
    },
    id::{BookId, UserId},
    list::PaginatedList,
};
use shared::error::AppResult;

#[mockall::automock]
#[async_trait]
pub trait BookRepository: Send + Sync {
    /// 書籍を登録する
    async fn create(&self, event: CreateBook, user_id: UserId) -> AppResult<()>;
    /// 書籍を全件取得する
    async fn find_all(&self, options: BookListOptions) -> AppResult<PaginatedList<Book>>;
    /// 書籍を取得する
    async fn find_by_id(&self, book_id: BookId) -> AppResult<Option<Book>>;
    /// 書籍を更新する
    async fn update(&self, event: UpdateBook) -> AppResult<()>;
    /// 書籍を削除する
    async fn delete(&self, event: DeleteBook) -> AppResult<()>;
}
