//! 蔵書の貸し出しのDB操作のための抽象実装をするモジュール
use async_trait::async_trait;

use crate::model::{
    checkout::{
        Checkout,
        event::{CreateCheckout, UpdateReturned},
    },
    id::{BookId, UserId},
};
use shared::error::AppResult;

#[async_trait]
pub trait CheckoutRepository: Send + Sync {
    /// 貸出操作を行う
    async fn create(&self, event: CreateCheckout) -> AppResult<()>;
    /// 返却操作を行う
    async fn update_returned(&self, event: UpdateReturned) -> AppResult<()>;
    /// 全ての未返却の貸出情報を取得する
    async fn find_unreturned_all(&self) -> AppResult<Vec<Checkout>>;
    /// ユーザーIDに紐づく未返却の貸出情報を取得する
    async fn find_unreturned_by_user_id(&self, user_id: UserId) -> AppResult<Vec<Checkout>>;
    /// 蔵書の貸出履歴（返却済みも含む）を取得する
    async fn find_history_by_book_id(&self, book_id: BookId) -> AppResult<Vec<Checkout>>;
}
