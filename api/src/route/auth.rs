use axum::{Router, routing::post};
use registry::AppRegistry;

use crate::handler::auth::{login, logout};

/// 認証関連のルータを作成する関数
pub fn build_auth_routers() -> Router<AppRegistry> {
    let auth_routers = Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout));

    Router::new().nest("/auth", auth_routers)
}
