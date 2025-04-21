use axum::{extract::State, http::StatusCode};
use registry::AppRegistry;

/// ヘルスチェック用のハンドラ
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

/// DBヘルスチェック用のハンドラ
pub async fn health_check_db(State(registry): State<AppRegistry>) -> StatusCode {
    // リポジトリの
    if registry.health_check_repository().check_db().await {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}
