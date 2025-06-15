use axum::{body::Body, http::Request};
use rstest::rstest;
use std::sync::Arc;
use tower::ServiceExt;

use crate::{
    deserialize_json,
    helper::{TestRequestExt, fixture, make_router, v1},
};

use api::model::book::PaginatedBookResponse;
use kernel::{
    model::{
        book::Book,
        id::{BookId, UserId},
        list::PaginatedList,
        user::BookOwner,
    },
    repository::book::MockBookRepository,
};

/// ・リクエストパスに応じて、期待している関数が返されることの確認
/// ・クエリパラメータのlimitとoffsetに応じて、モックの戻り値が変わることの確認
#[rstest]
#[case("/books", 20, 0)]
#[case("/books?limit=50", 50, 0)]
#[case("/books?limit=50&offset=20", 50, 20)]
#[case("/books?offset=20", 20, 20)]
#[tokio::test]
async fn test_show_book_list_with_query_200(
    // helper.rsで定義したfixture関数を使用（関数の実行結果がfixtureとして渡される）
    mut fixture: registry::MockAppRegistryExt,
    #[case] path: &str,
    #[case] expected_limit: i64,
    #[case] expected_offset: i64,
) -> anyhow::Result<()> {
    let book_id = BookId::new();

    // モックの設定
    fixture.expect_book_repository().returning(move || {
        let mut mock = MockBookRepository::new();

        // optionsはfind_allの引数
        // queryで与えられたlimitとoffsetを使用して、モックの戻り値を設定
        mock.expect_find_all().returning(move |options| {
            let items = vec![Book {
                id: book_id,
                title: "Test Book".to_string(),
                isbn: "".to_string(),
                author: "Test Author".to_string(),
                description: "Test Description".to_string(),
                owner: BookOwner {
                    id: UserId::new(),
                    name: "Test User".to_string(),
                },
                checkout: None,
            }];
            Ok(PaginatedList {
                total: 1,
                limit: options.limit,
                offset: options.offset,
                items,
            })
        });
        Arc::new(mock)
    });

    // ルーターを作成
    let router: axum::Router = make_router(fixture);

    // リクエストを作成・送信し、レスポンスを検証
    let request = Request::get(&v1(path)).bearer().body(Body::empty())?;
    let resp = router.oneshot(request).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::OK);

    // レスポンスの値を検証
    let result = deserialize_json!(resp, PaginatedBookResponse);
    assert_eq!(result.limit, expected_limit);
    assert_eq!(result.offset, expected_offset);

    Ok(())
}

/// limitやoffsetが不正な値の場合の異常系テスト
#[rstest]
#[case("/books?limit=-1")]
#[case("/books?offset=-10")]
#[case("/books?limit=abc")]
#[case("/books?offset=xyz")]
#[tokio::test]
async fn test_show_book_list_with_query_400(
    mut fixture: registry::MockAppRegistryExt,
    #[case] path: &str,
) -> anyhow::Result<()> {
    // モックの設定（呼ばれないことを期待）
    fixture.expect_book_repository().returning(|| {
        let mut mock = MockBookRepository::new();
        mock.expect_find_all().never();
        Arc::new(mock)
    });

    let router: axum::Router = make_router(fixture);

    let request = Request::get(&v1(path)).bearer().body(Body::empty())?;
    let resp = router.oneshot(request).await?;
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);

    Ok(())
}
