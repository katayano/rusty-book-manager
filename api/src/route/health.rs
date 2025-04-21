use axum::{Router, routing::get};
use registry::AppRegistry;

use crate::handler::health::{health_check, health_check_db};

/// ヘルスチェック用のルータを作成する関数
pub fn build_health_check_routers() -> Router<AppRegistry> {
    let routers = Router::new()
        .route("/", get(health_check))
        .route("/db", get(health_check_db));
    // ヘルスチェックに関連するパスのルートである/healthに個別のパスをネストする
    Router::new().nest("/health", routers)
}
