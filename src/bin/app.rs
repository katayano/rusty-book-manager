use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use adapter::{database::connect_database_with, redis::RedisClient};
use anyhow::{Context, Result};
#[cfg(debug_assertions)]
use api::openapi::ApiDoc;
use api::route::{auth::build_auth_routers, v1};
use axum::{Router, http::Method};
use opentelemetry::global;
use registry::AppRegistryImpl;
use shared::config::AppConfig;
use shared::env::{Environment, which};
use tokio::net::TcpListener;
use tower_http::LatencyUnit;
use tower_http::cors::{self, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;
#[cfg(not(debug_assertions))]
use tracing::subscriber;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
#[cfg(debug_assertions)]
use utoipa::OpenApi;
#[cfg(debug_assertions)]
use utoipa_redoc::{Redoc, Servable};

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

    // 環境変数の読み込み
    let host = std::env::var("JAEGER_HOST")?;
    let port = std::env::var("JAEGER_PORT")?;
    let endpoint = format!("{host}:{port}");

    global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        // 以下、tracerの設定
        .with_endpoint(endpoint)
        .with_service_name("book-manager")
        .with_auto_split_batch(true)
        // このくらいのbytesを送れればいいという値
        // 足りないとエラーになる
        .with_max_packet_size(8192)
        .install_simple()?;

    // tracingとopentelemetryをブリッジさせる設定(layer)
    let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // ログレベルを設定
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| log_level.into());

    // ログのフォーマットを設定
    let subscriber = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_target(false);

    #[cfg(not(debug_assertions))]
    // リリースビルドではJSON形式でログを出力
    let subscriber = subscriber.json();

    // ロガーを初期化
    tracing_subscriber::registry()
        .with(subscriber)
        .with(env_filter)
        .with(opentelemetry)
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
    // AppRegistry(DIコンテナ)を作成
    let registry = Arc::new(AppRegistryImpl::new(pool, kv, app_config));

    // ヘルスチェック用のルーターを作成
    // ルーターのStateにAppRegistryを登録し、各ハンドラで使えるようにする
    let app = Router::new()
        .merge(v1::routes())
        .merge(build_auth_routers())
        // Redocのドキュメントが生成される
        .merge(Redoc::with_url("/docs", ApiDoc::openapi()))
        // CORSの設定
        .layer(cors())
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
        .with_graceful_shutdown(shutdown_signal())
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

fn cors() -> CorsLayer {
    CorsLayer::new()
        .allow_headers(cors::Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(cors::Any)
}

async fn shutdown_signal() {
    fn purge_spans() {
        global::shutdown_tracer_provider();
    }

    // Ctrl-Cで終了した際にトレースをパージする
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl-C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install terminate handler")
            .recv()
            .await
            .expect("Failed to receive terminate signal");
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl-C, shutting down...");
            purge_spans();
        }
        _ = terminate => {
            tracing::info!("Received terminate signal, shutting down...");
            purge_spans();
        }
    }
}
