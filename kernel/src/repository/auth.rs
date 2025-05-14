use async_trait::async_trait;

use crate::model::{
    auth::{AccessToken, event::CreateToken},
    id::UserId,
};
use shared::error::AppResult;

#[async_trait]
pub trait AuthRepository: Send + Sync {
    /// アクセストークンからユーザーIDを取得する
    async fn fetch_user_id_from_token(
        &self,
        access_token: &AccessToken,
    ) -> AppResult<Option<UserId>>;
    /// メールアドレスとパスワードが正しいか検証する
    async fn verify_user(&self, email: &str, password: &str) -> AppResult<UserId>;
    /// アクセストークンを生成する
    async fn create_token(&self, event: CreateToken) -> AppResult<AccessToken>;
    /// アクセストークンを削除する
    async fn delete_token(&self, access_token: AccessToken) -> AppResult<()>;
}
