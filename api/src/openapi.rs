use crate::{handler, model};

#[derive(utoipa::OpenApi)]
#[openapi(
    // メタ情報を定義
    info(
        title = "Book App",
        contact(
            name = "RustによるWebアプリケーション開発",
            url = "todo",
            email = "todo"
        ),
        description = "Rustの勉強",
    ),
    // OpenAPI定義上で使用するパスを指定
    paths(
        // handler::health::health_check,
        // handler::health::health_check_db,
        handler::book::register_book,
        handler::book::show_book_list,
        // handler::book::show_book,
        // handler::book::update_book,
        // handler::book::delete_book,
        // handler::checkout::checkout_book,
        // handler::checkout::return_book,
        // handler::checkout::checkout_history,
        // handler::user::get_current_user,
        // handler::auth::login,
        // handler::auth::logout,
    ),
    // OpenAPIドキュメント内で使用するコンポーネントを定義
    components(schemas(
        model::book::CreateBookRequest,
        model::book::UpdateBookRequest,
        model::book::BookResponse,
        model::book::PaginatedBookResponse,
        model::book::BookCheckoutResponse,
        model::checkout::CheckoutsResponse,
        model::checkout::CheckoutResponse,
        model::checkout::CheckoutBookResponse,
        model::user::BookOwner,
        model::user::CheckoutUser,
        model::auth::LoginRequest,
        model::auth::AccessTokenResponse,
    ))
)]
pub struct ApiDoc;
