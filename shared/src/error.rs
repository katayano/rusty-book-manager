use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    UnprocessableEntity(String),
    #[error("{0}")]
    EntityNotFound(String),
    #[error("{0}")]
    ValidationError(#[from] garde::Report),
    #[error("トランザクションを実行できませんでした")]
    TransactionError(#[source] sqlx::Error),
    #[error("データベース処理実行中にエラーが発生しました")]
    SpecificOperationError(#[source] sqlx::Error),
    #[error("No rows affected: {0}")]
    NoRowsAffectedError(String),
    #[error("{0}")]
    KeyValueStoreError(#[from] redis::RedisError),
    #[error("{0}")]
    BcryptError(#[from] bcrypt::BcryptError),
    #[error("{0}")]
    ConvertToUuidError(#[from] uuid::Error),
    #[error("ログインに失敗しました")]
    UnauthenticatedError,
    #[error("認可情報が誤っています")]
    UnauthorizedError,
    #[error("許可されていない操作です")]
    ForbiddenOperationError,
    #[error("{0}")]
    ConversionEntityError(String),
}

/// Errorをレスポンスに変換するためのトレイト
impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        // エラーの種類に応じて、適切なHTTPステータスコードを返す
        let status_code = match self {
            AppError::UnprocessableEntity(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            AppError::ValidationError(_) | AppError::ConvertToUuidError(_) => {
                StatusCode::BAD_REQUEST
            }
            AppError::UnauthenticatedError | AppError::ForbiddenOperationError => {
                StatusCode::FORBIDDEN
            }
            AppError::UnauthorizedError => StatusCode::UNAUTHORIZED,
            e @ (AppError::TransactionError(_)
            | AppError::SpecificOperationError(_)
            | AppError::NoRowsAffectedError(_)
            | AppError::KeyValueStoreError(_)
            | AppError::BcryptError(_)
            | AppError::ConversionEntityError(_)) => {
                tracing::error!(
                                    error.cause_chain = ?e,
                                    error.message = %e,
                                    "Unexpected error occurred"
                );
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        status_code.into_response()
    }
}

/// エラー型がAppErrorなものをとらえるResult型
pub type AppResult<T> = Result<T, AppError>;
