use std::sync::Arc;

use adapter::{
    database::ConnectionPool,
    redis::RedisClient,
    repository::{
        auth::AuthRepositoryImpl, book::BookRepositoryImpl, checkout::CheckoutRepositoryImpl,
        health::HealthCheckRepositoryImpl, user::UserRepositoryImpl,
    },
};
use kernel::repository::{
    auth::AuthRepository, book::BookRepository, checkout::CheckoutRepository,
    health::HealthCheckRepository, user::UserRepository,
};
use shared::config::AppConfig;

/// DIコンテナの構造体
#[derive(Clone)]
pub struct AppRegistryImpl {
    health_check_repository: Arc<dyn HealthCheckRepository>,
    book_repository: Arc<dyn BookRepository>,
    auth_repository: Arc<dyn AuthRepository>,
    user_repository: Arc<dyn UserRepository>,
    checkout_repository: Arc<dyn CheckoutRepository>,
}

impl AppRegistryImpl {
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
        let checkout_repository = Arc::new(CheckoutRepositoryImpl::new(db.clone()));
        Self {
            health_check_repository,
            book_repository,
            auth_repository,
            user_repository,
            checkout_repository,
        }
    }
}

#[mockall::automock]
pub trait AppRegistryExt {
    /// ヘルスチェックリポジトリを取得する
    fn health_check_repository(&self) -> Arc<dyn HealthCheckRepository>;
    /// 書籍リポジトリを取得する
    fn book_repository(&self) -> Arc<dyn BookRepository>;
    /// 認証リポジトリを取得する
    fn auth_repository(&self) -> Arc<dyn AuthRepository>;
    /// ユーザリポジトリを取得する
    fn user_repository(&self) -> Arc<dyn UserRepository>;
    /// 貸出リポジトリを取得する
    fn checkout_repository(&self) -> Arc<dyn CheckoutRepository>;
}

impl AppRegistryExt for AppRegistryImpl {
    fn health_check_repository(&self) -> Arc<dyn HealthCheckRepository> {
        self.health_check_repository.clone()
    }

    fn book_repository(&self) -> Arc<dyn BookRepository> {
        self.book_repository.clone()
    }

    fn auth_repository(&self) -> Arc<dyn AuthRepository> {
        self.auth_repository.clone()
    }

    fn user_repository(&self) -> Arc<dyn UserRepository> {
        self.user_repository.clone()
    }

    fn checkout_repository(&self) -> Arc<dyn CheckoutRepository> {
        self.checkout_repository.clone()
    }
}

pub type AppRegistry = Arc<dyn AppRegistryExt + Send + Sync + 'static>;
