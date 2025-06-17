use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use garde::Validate;

use kernel::model::{book::event::DeleteBook, id::BookId};
use registry::AppRegistry;
use shared::error::{AppError, AppResult};

use crate::{
    extractor::AuthorizedUser,
    model::book::{
        BookListQuery, BookResponse, CreateBookRequest, PaginatedBookResponse, UpdateBookRequest,
        UpdateBookRequestWithIds,
    },
};

/// 書籍を登録するハンドラ
/// アクセストークンによるユーザー認証を行うために、AuthorizedUserを引数に取る
pub async fn register_book(
    user: AuthorizedUser,
    State(registry): State<AppRegistry>,
    Json(req): Json<CreateBookRequest>,
) -> AppResult<StatusCode> {
    req.validate()?;

    registry
        .book_repository()
        .create(req.into(), user.id())
        .await
        .map(|_| StatusCode::CREATED)
}

/// 書籍を取得するハンドラ
pub async fn show_book_list(
    _user: AuthorizedUser,
    Query(query): Query<BookListQuery>,
    State(registry): State<AppRegistry>,
) -> AppResult<Json<PaginatedBookResponse>> {
    query.validate()?;

    registry
        .book_repository()
        .find_all(query.into())
        .await
        .map(|books| books.into())
        .map(Json)
}

/// IDに一致する書籍を取得するハンドラ
#[tracing::instrument(
    skip(_user, registry),
    fields(
        user_id = %_user.user.id.to_string()
    )
)]
pub async fn show_book(
    _user: AuthorizedUser,
    State(registry): State<AppRegistry>,
    Path(book_id): Path<BookId>,
) -> AppResult<Json<BookResponse>> {
    tracing::info!("ログを追加");
    registry
        .book_repository()
        .find_by_id(book_id)
        .await
        .and_then(|bc| match bc {
            Some(bc) => Ok(Json(bc.into())),
            None => Err(AppError::EntityNotFound("not found".into())),
        })
}

/// 書籍を更新するハンドラ
pub async fn update_book(
    user: AuthorizedUser,
    State(registry): State<AppRegistry>,
    Path(book_id): Path<BookId>,
    Json(req): Json<UpdateBookRequest>,
) -> AppResult<StatusCode> {
    req.validate()?;

    let update_book = UpdateBookRequestWithIds::new(book_id, user.id(), req);

    registry
        .book_repository()
        .update(update_book.into())
        .await
        .map(|_| StatusCode::OK)
}

/// 書籍を削除するハンドラ
pub async fn delete_book(
    user: AuthorizedUser,
    State(registry): State<AppRegistry>,
    Path(book_id): Path<BookId>,
) -> AppResult<StatusCode> {
    let delete_book = DeleteBook {
        book_id,
        requested_user: user.id(),
    };

    registry
        .book_repository()
        .delete(delete_book)
        .await
        .map(|_| StatusCode::OK)
}
