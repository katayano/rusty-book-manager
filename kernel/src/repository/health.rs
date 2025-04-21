//! ヘルスチェックのDB操作のための抽象実装をするモジュール

use async_trait::async_trait;

#[async_trait]
pub trait HealthCheckRepository: Send + Sync {
    /// DB接続の確認を行う関数
    async fn check_db(&self) -> bool;
}
