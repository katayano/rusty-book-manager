use axum::Router;
use registry::AppRegistry;

use super::{
    book::build_book_routers, health::build_health_check_routers, user::build_user_routers,
};

/// v1 APIのルータを構築する関数
pub fn routes() -> Router<AppRegistry> {
    let router = Router::new()
        .merge(build_book_routers())
        .merge(build_health_check_routers())
        .merge(build_user_routers());

    Router::new().nest("/api/v1", router)
}
