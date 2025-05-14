use shared::error::{AppError, AppResult};
use std::str::FromStr;

use crate::redis::model::{RedisKey, RedisValue};
use kernel::model::{
    auth::{AccessToken, event::CreateToken},
    id::UserId,
};

pub struct UserItem {
    pub user_id: UserId,
    pub password_hash: String,
}

/// Redisに保存するためのKey
pub struct AuthorizationKey(String);
/// Redisに保存するためのValue
pub struct AuthorizedUserId(UserId);

pub fn from(event: CreateToken) -> (AuthorizationKey, AuthorizedUserId) {
    (
        AuthorizationKey(event.access_token),
        AuthorizedUserId(event.user_id),
    )
}

impl From<AuthorizationKey> for AccessToken {
    fn from(key: AuthorizationKey) -> Self {
        AccessToken(key.0)
    }
}

impl From<AccessToken> for AuthorizationKey {
    fn from(token: AccessToken) -> Self {
        AuthorizationKey(token.0)
    }
}

impl From<&AccessToken> for AuthorizationKey {
    fn from(token: &AccessToken) -> Self {
        AuthorizationKey(token.0.to_string())
    }
}

/* 以下の実装群はRedisKeyとRedisValueの実装 */

impl RedisKey for AuthorizationKey {
    type Value = AuthorizedUserId;

    fn inner(&self) -> String {
        self.0.clone()
    }
}

impl RedisValue for AuthorizedUserId {
    fn inner(&self) -> String {
        self.0.to_string()
    }
}

impl TryFrom<String> for AuthorizedUserId {
    type Error = AppError;

    fn try_from(value: String) -> AppResult<Self> {
        Ok(Self(UserId::from_str(&value).map_err(|e| {
            AppError::ConversionEntityError(e.to_string())
        })?))
    }
}

impl AuthorizedUserId {
    pub fn into_inner(self) -> UserId {
        self.0
    }
}
