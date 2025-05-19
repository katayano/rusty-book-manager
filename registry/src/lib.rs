use std::sync::Arc;

use adapter::{
    database::ConnectionPool,
    redis::RedisClient,
    repository::{
        auth::AuthRepositoryImpl, book::BookRepositoryImpl, health::HealthCheckRepositoryImpl,
        user::UserRepositoryImpl,
    },
};
use kernel::repository::{
    auth::AuthRepository, book::BookRepository, health::HealthCheckRepository, user::UserRepository,
};
use shared::config::AppConfig;

/// DIコンテナの構造体
#[derive(Clone)]
pub struct AppRegistry {
    health_check_repository: Arc<dyn HealthCheckRepository>,
    book_repository: Arc<dyn BookRepository>,
    auth_repository: Arc<dyn AuthRepository>,
    user_repository: Arc<dyn UserRepository>,
}

impl AppRegistry {
    /// DIコンテナを作成する
    pub fn new(db: ConnectionPool, redis_client: Arc<RedisClient>, app_config: AppConfig) -> Self {
        let health_check_repository = Arc::new(HealthCheckRepositoryImpl::new(db.clone()));
        let book_repository = Arc::new(BookRepositoryImpl::new(db.clone()));
        let auth_repository = Arc::new(AuthRepositoryImpl::new(
            db.clone(),
            redis_client.clone(),
            app_config.auth.ttl,
        ));
        let user_repository = Arc::new(UserRepositoryImpl::new(db.clone()));
        Self {
            health_check_repository,
            book_repository,
            auth_repository,
            user_repository,
        }
    }

    /// ヘルスチェックリポジトリを取得する
    pub fn health_check_repository(&self) -> Arc<dyn HealthCheckRepository> {
        self.health_check_repository.clone()
    }

    /// 書籍リポジトリを取得する
    pub fn book_repository(&self) -> Arc<dyn BookRepository> {
        self.book_repository.clone()
    }
    /// 認証リポジトリを取得する
    pub fn auth_repository(&self) -> Arc<dyn AuthRepository> {
        self.auth_repository.clone()
    }
    /// ユーザリポジトリを取得する
    pub fn user_repository(&self) -> Arc<dyn UserRepository> {
        self.user_repository.clone()
    }
}
