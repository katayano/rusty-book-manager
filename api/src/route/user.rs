use axum::{
    Router,
    routing::{delete, get, put},
};

use registry::AppRegistry;

use crate::handler::user::{
    delete_user, get_checkouts, get_current_user, list_users, register_user, update_user_password,
    update_user_role,
};

/// ユーザー関連のルータを作成する関数
pub fn build_user_routers() -> Router<AppRegistry> {
    Router::new()
        .route("/users/me", get(get_current_user))
        .route("/users/me/password", put(update_user_password))
        .route("/users", get(list_users).post(register_user))
        .route("/users/{user_id}", delete(delete_user))
        .route("/users/{user_id}/role", put(update_user_role))
        .route("/users/me/checkouts", get(get_checkouts))
}
