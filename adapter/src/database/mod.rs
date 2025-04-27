//! DB接続を行うモジュール

use shared::config::DatabaseConfig;
use sqlx::{PgPool, postgres::PgConnectOptions};

pub mod model;

/// DatabaseConfigからPostgres接続用の構造体へ変換
fn make_pg_connect_options(cfg: &DatabaseConfig) -> PgConnectOptions {
    PgConnectOptions::new()
        .host(&cfg.host)
        .port(cfg.port)
        .username(&cfg.username)
        .password(&cfg.password)
        .database(&cfg.database)
}

/// sqlx::PgPoolをラップする構造体
/// ラップすることで、sqlx::PgPoolのメソッドを隠蔽し、
/// 呼出し元でsqlx::PgPoolの依存の追加が不要
#[derive(Clone)]
pub struct ConnectionPool(PgPool);

impl ConnectionPool {
    pub fn new(pool: PgPool) -> Self {
        Self(pool)
    }
    /// sqlx::PgPoolを取得する
    pub fn inner_ref(&self) -> &PgPool {
        &self.0
    }
}

///  コネクションプールを作成
pub fn connect_database_with(cfg: &DatabaseConfig) -> ConnectionPool {
    ConnectionPool(PgPool::connect_lazy_with(make_pg_connect_options(cfg)))
}
