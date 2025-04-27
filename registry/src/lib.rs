use std::sync::Arc;

use adapter::database::ConnectionPool;
use adapter::repository::{book::BookRepositoryImpl, health::HealthCheckRepositoryImpl};
use kernel::repository::book::BookRepository;
use kernel::repository::health::HealthCheckRepository;

/// DIコンテナの構造体
#[derive(Clone)]
pub struct AppRegistry {
    health_check_repository: Arc<dyn HealthCheckRepository>,
    book_repository: Arc<dyn BookRepository>,
}

impl AppRegistry {
    /// DIコンテナを作成する
    pub fn new(db: ConnectionPool) -> Self {
        let health_check_repository = Arc::new(HealthCheckRepositoryImpl::new(db.clone()));
        let book_repository = Arc::new(BookRepositoryImpl::new(db.clone()));
        Self {
            health_check_repository,
            book_repository,
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
}
