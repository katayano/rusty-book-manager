//! ヘルスチェックのDB操作のための具象実装をするモジュール

use async_trait::async_trait;
use derive_new::new;
use kernel::repository::health::HealthCheckRepository;

use crate::database::ConnectionPool;

#[derive(new)]
pub struct HealthCheckRepositoryImpl {
    db: ConnectionPool,
}

#[async_trait]
impl HealthCheckRepository for HealthCheckRepositoryImpl {
    async fn check_db(&self) -> bool {
        sqlx::query("SELECT 1")
            .fetch_one(self.db.inner_ref())
            .await
            .is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn test_check_db(pool: sqlx::PgPool) {
        // HealthCheckRepositoryImplを初期化
        let repository = HealthCheckRepositoryImpl::new(ConnectionPool::new(pool));

        let check_result = repository.check_db().await;
        assert!(check_result);
    }
}
