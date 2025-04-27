use std::net::{Ipv4Addr, SocketAddr};

use adapter::database::connect_database_with;
use anyhow::{Error, Result};
use api::route::book::build_book_routers;
use api::route::health::build_health_check_routers;
use axum::Router;
use registry::AppRegistry;
use shared::config::AppConfig;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    bootstrap().await
}

/// サーバー起動の関数
async fn bootstrap() -> Result<()> {
    // AppConfigを初期化
    let app_config = AppConfig::new()?;
    // コネクションプールを作成
    let pool = connect_database_with(&app_config.database);
    // AppRegitry(DIコンテナ)を作成
    let registry = AppRegistry::new(pool);

    // ヘルスチェック用のルーターを作成
    // ルーターのStateにAppRegistryを登録し、各ハンドラで使えるようにする
    let app = Router::new()
        .merge(build_health_check_routers())
        .merge(build_book_routers())
        .with_state(registry);

    // サーバーを起動
    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8080);
    let listener = TcpListener::bind(addr).await?;

    println!("Listening on {}", addr);

    axum::serve(listener, app).await.map_err(Error::from)
}

// #[tokio::test]
// async fn health_check_works() {
//     // awaitして結果を得る
//     let status_code = health_check().await;
//     assert_eq!(status_code, StatusCode::OK);
// }

// #[sqlx::test]
// async fn health_check_db_works(pool: sqlx::PgPool) {
//     let status_code = health_check_db(State(pool)).await;
//     assert_eq!(status_code, StatusCode::OK);
// }
