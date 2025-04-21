use std::sync::Arc;

use adapter::{database::ConnectionPool, repository::health::HealthCheckRepositoryImpl};
use kernel::repository::health::HealthCheckRepository;

/// DIコンテナの構造体
#[derive(Clone)]
pub struct AppRegistry {
    health_check_repository: Arc<dyn HealthCheckRepository>,
}

impl AppRegistry {
    /// DIコンテナを作成する
    pub fn new(db: ConnectionPool) -> Self {
        let health_check_repository = Arc::new(HealthCheckRepositoryImpl::new(db));
        Self {
            health_check_repository,
        }
    }

    /// ヘルスチェックリポジトリを取得する
    pub fn health_check_repository(&self) -> Arc<dyn HealthCheckRepository> {
        self.health_check_repository.clone()
    }
}
