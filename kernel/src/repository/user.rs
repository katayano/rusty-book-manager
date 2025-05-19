//! ユーザー関連の操作をするための抽象実装をするモジュール
use async_trait::async_trait;

use crate::model::{
    id::UserId,
    user::{
        User,
        event::{CreateUser, DeleteUser, UpdateUserPassword, UpdateUserRole},
    },
};
use shared::error::AppResult;

#[async_trait]
pub trait UserRepository: Send + Sync {
    /// ユーザーを登録する
    async fn create(&self, event: CreateUser) -> AppResult<User>;
    /// ユーザーを全件取得する
    async fn find_all(&self) -> AppResult<Vec<User>>;
    /// ユーザーを取得する
    async fn find_current_user(&self, current_user_id: UserId) -> AppResult<Option<User>>;
    /// ユーザーを削除する
    async fn delete(&self, event: DeleteUser) -> AppResult<()>;
    /// ユーザーのロールを更新する
    async fn update_role(&self, event: UpdateUserRole) -> AppResult<()>;
    /// ユーザーのパスワードを更新する
    async fn update_password(&self, event: UpdateUserPassword) -> AppResult<()>;
}
