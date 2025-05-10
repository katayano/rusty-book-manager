//! 蔵書のDB操作のための抽象実装をするモジュール
use async_trait::async_trait;

use crate::model::{
    book::{Book, event::CreateBook},
    id::BookId,
};
use shared::error::AppResult;

#[async_trait]
pub trait BookRepository: Send + Sync {
    /// 書籍を登録する
    async fn create(&self, event: CreateBook) -> AppResult<()>;
    /// 書籍を全件取得する
    async fn find_all(&self) -> AppResult<Vec<Book>>;
    /// 書籍を取得する
    async fn find_by_id(&self, book_id: BookId) -> AppResult<Option<Book>>;
}
