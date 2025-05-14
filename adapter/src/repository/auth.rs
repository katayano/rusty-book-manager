use std::sync::Arc;

use async_trait::async_trait;
use derive_new::new;
use kernel::{
    model::{
        auth::{AccessToken, event::CreateToken},
        id::UserId,
    },
    repository::auth::AuthRepository,
};
use shared::error::{AppError, AppResult};

use crate::{
    database::{
        ConnectionPool,
        model::auth::{AuthorizationKey, AuthorizedUserId, UserItem, from},
    },
    redis::RedisClient,
};

#[derive(new)]
pub struct AuthRepositoryImpl {
    db: ConnectionPool,
    kv: Arc<RedisClient>,
    ttl: u64,
}

#[async_trait]
impl AuthRepository for AuthRepositoryImpl {
    /// アクセストークンからユーザーIDを取得する
    async fn fetch_user_id_from_token(
        &self,
        access_token: &AccessToken,
    ) -> AppResult<Option<UserId>> {
        let key = AuthorizationKey::from(access_token);
        self.kv
            .get(&key)
            .await
            .map(|x| x.map(AuthorizedUserId::into_inner))
    }

    /// メールアドレスとパスワードが正しいか検証する
    async fn verify_user(&self, email: &str, password: &str) -> AppResult<UserId> {
        let user_item = sqlx::query_as!(
            UserItem,
            r#"
            SELECT user_id, password_hash
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_one(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        // パスワードの検証
        let valid = bcrypt::verify(password, &user_item.password_hash)?;
        if !valid {
            return Err(AppError::UnauthenticatedError);
        }

        Ok(user_item.user_id)
    }

    /// アクセストークンを生成する
    async fn create_token(&self, event: CreateToken) -> AppResult<AccessToken> {
        let (key, value) = from(event);
        self.kv.set_ex(&key, &value, self.ttl).await?;

        Ok(key.into())
    }

    /// アクセストークンを削除する
    async fn delete_token(&self, access_token: AccessToken) -> AppResult<()> {
        let key = AuthorizationKey::from(access_token);
        self.kv.delete(&key).await
    }
}
