use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use adapter::{database::connect_database_with, redis::RedisClient};
use anyhow::{Context, Result};
use api::route::{auth::build_auth_routers, v1};
use axum::Router;
use registry::AppRegistry;
use shared::config::AppConfig;
use shared::env::{Environment, which};
use tokio::net::TcpListener;
use tower_http::LatencyUnit;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() -> Result<()> {
    init_logger()?;
    bootstrap().await
}

/// ロガーを初期化する関数
fn init_logger() -> Result<()> {
    let log_level = match which() {
        Environment::Development => "debug",
        Environment::Production => "info",
    };
    // ログレベルを設定
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| log_level.into());

    // ログのフォーマットを設定
    let subscriber = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_target(false);

    // ロガーを初期化
    tracing_subscriber::registry()
        .with(subscriber)
        .with(env_filter)
        .try_init()?;
    Ok(())
}

/// サーバー起動の関数
async fn bootstrap() -> Result<()> {
    // AppConfigを初期化
    let app_config = AppConfig::new()?;
    // コネクションプールを作成
    let pool = connect_database_with(&app_config.database);
    // Redisクライアントを作成
    let kv = Arc::new(RedisClient::new(&app_config.redis)?);
    // AppRegitry(DIコンテナ)を作成
    let registry = AppRegistry::new(pool, kv, app_config);

    // ヘルスチェック用のルーターを作成
    // ルーターのStateにAppRegistryを登録し、各ハンドラで使えるようにする
    let app = Router::new()
        .merge(v1::routes())
        .merge(build_auth_routers())
        // リクエストとレスポンス時にログ出力するレイヤーを追加
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Micros),
                ),
        )
        .with_state(registry);

    // サーバーを起動
    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8080);
    let listener = TcpListener::bind(addr).await?;

    tracing::info!("Listening on {}", addr);

    axum::serve(listener, app)
        .await
        .context("Unexpected error occurred in server")
        // 起動失敗した際のエラーログを出力
        .inspect_err(|e| {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "Unexpected error"
            )
        })
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
