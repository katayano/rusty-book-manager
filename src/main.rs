use std::net::{Ipv4Addr, SocketAddr};

use ::axum::{Router, http::StatusCode, routing::get};
use anyhow::Result;
use tokio::net::TcpListener;

// ヘルスチェック用のハンドラ
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

#[tokio::main]
async fn main() -> Result<()> {
    // ヘルスチェック用のルーター
    let app = Router::new().route("/health", get(health_check));
    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8080);
    let listener = TcpListener::bind(addr).await?;

    println!("Listening on {}", addr);

    Ok(axum::serve(listener, app).await?)
}

#[tokio::test]
async fn health_check_works() {
    // awaitして結果を得る
    let status_code = health_check().await;
    assert_eq!(status_code, StatusCode::OK);
}
