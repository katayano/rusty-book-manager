//! 蔵書のDB操作のための抽象実装をするモジュール
use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

use crate::model::book::{Book, event::CreateBook};

#[async_trait]
pub trait BookRepository: Send + Sync {
    /// 書籍を登録する
    async fn create(&self, event: CreateBook) -> Result<()>;
    /// 書籍を全件取得する
    async fn find_all(&self) -> Result<Vec<Book>>;
    /// 書籍を取得する
    async fn find_by_id(&self, book_id: Uuid) -> Result<Option<Book>>;
}
