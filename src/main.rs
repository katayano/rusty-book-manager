use std::net::{Ipv4Addr, SocketAddr};

use ::axum::{Router, routing::get};
use tokio::net::TcpListener;

// ハンドラ
// どのリクエストでもHello worldを返す
async fn hello_world() -> &'static str {
    "Hello world!"
}

// tokioランタイム上で動かすために必要なマクロ
// このマクロを使用すると、main関数を非同期化できる
#[tokio::main]
async fn main() {
    // ルーター
    let app = Router::new().route("/hello", get(hello_world));
    // 8080ポートでリクエストを待ち受ける
    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8080);
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Listening on {}", addr);
    // サーバーを起動
    axum::serve(listener, app).await.unwrap();
}
