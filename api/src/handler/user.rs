use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use garde::Validate;

use kernel::model::{id::UserId, user::event::DeleteUser};
use registry::AppRegistry;
use shared::error::{AppError, AppResult};

use crate::{
    extractor::AuthorizedUser,
    model::checkout::CheckoutsResponse,
    model::user::{
        CreateUserRequest, UpdateUserPasswordRequest, UpdateUserPasswordRequestWithUserId,
        UpdateUserRoleRequest, UpdateUserRoleRequestWithUserId, UserResponse, UsersResponse,
    },
};

/// ユーザーを登録するハンドラ（管理者のみ）
pub async fn register_user(
    user: AuthorizedUser,
    State(registry): State<AppRegistry>,
    Json(req): Json<CreateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    // 管理者のみがユーザーを登録できる
    if !user.is_admin() {
        return Err(AppError::ForbiddenOperationError);
    }

    req.validate()?;

    let registered_user = registry.user_repository().create(req.into()).await?;

    Ok(Json(UserResponse::from(registered_user)))
}

/// ユーザーを全件取得するハンドラ
pub async fn list_users(
    _user: AuthorizedUser,
    State(registry): State<AppRegistry>,
) -> AppResult<Json<UsersResponse>> {
    let users = registry
        .user_repository()
        .find_all()
        .await?
        .into_iter()
        .map(UserResponse::from)
        .collect();

    Ok(Json(UsersResponse { users }))
}

/// ユーザーを削除するハンドラ（管理者のみ）
pub async fn delete_user(
    user: AuthorizedUser,
    State(registry): State<AppRegistry>,
    Path(user_id): Path<UserId>,
) -> AppResult<StatusCode> {
    // 管理者のみがユーザーを削除できる
    if !user.is_admin() {
        return Err(AppError::ForbiddenOperationError);
    }

    registry
        .user_repository()
        .delete(DeleteUser { id: user_id })
        .await?;

    Ok(StatusCode::OK)
}

/// ユーザーのロールを変更するハンドラ（管理者のみ）
pub async fn update_user_role(
    user: AuthorizedUser,
    State(registry): State<AppRegistry>,
    Path(user_id): Path<UserId>,
    Json(req): Json<UpdateUserRoleRequest>,
) -> AppResult<StatusCode> {
    // 管理者のみがユーザーのロールを変更できる
    if !user.is_admin() {
        return Err(AppError::ForbiddenOperationError);
    }

    registry
        .user_repository()
        .update_role(UpdateUserRoleRequestWithUserId::new(user_id, req).into())
        .await?;

    Ok(StatusCode::OK)
}

/// ユーザーが自分自身のユーザー情報を取得するハンドラ
pub async fn get_current_user(user: AuthorizedUser) -> Json<UserResponse> {
    Json(UserResponse::from(user.user))
}

/// ユーザーのパスワードを変更するハンドラ
pub async fn update_user_password(
    user: AuthorizedUser,
    State(registry): State<AppRegistry>,
    Json(req): Json<UpdateUserPasswordRequest>,
) -> AppResult<StatusCode> {
    req.validate()?;

    registry
        .user_repository()
        .update_password(UpdateUserPasswordRequestWithUserId::new(user.id(), req).into())
        .await?;

    Ok(StatusCode::OK)
}

/// ユーザーが借りている蔵書一覧を取得するハンドラ
pub async fn get_checkouts(
    user: AuthorizedUser,
    State(registry): State<AppRegistry>,
) -> AppResult<Json<CheckoutsResponse>> {
    registry
        .checkout_repository()
        .find_unreturned_by_user_id(user.id())
        .await
        .map(|checkouts| CheckoutsResponse::from(checkouts))
        .map(Json)
}
