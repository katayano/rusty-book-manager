//! アプリケーション全体で使用する設定を定義する

use anyhow::Result;

pub struct AppConfig {
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub auth: AuthConfig,
}

impl AppConfig {
    pub fn new() -> Result<Self> {
        let database = DatabaseConfig {
            host: std::env::var("DATABASE_HOST")?,
            port: std::env::var("DATABASE_PORT")?.parse()?,
            username: std::env::var("DATABASE_USERNAME")?,
            password: std::env::var("DATABASE_PASSWORD")?,
            database: std::env::var("DATABASE_NAME")?,
        };
        let redis = RedisConfig {
            host: std::env::var("REDIS_HOST")?,
            port: std::env::var("REDIS_PORT")?.parse::<u16>()?,
        };
        let auth = AuthConfig {
            ttl: std::env::var("AUTH_TOKEN_TTL")?.parse::<u64>()?,
        };
        Ok(Self {
            database,
            redis,
            auth,
        })
    }
}

// DB接続設定を表す構造体
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

// Redis接続設定を表す構造体
pub struct RedisConfig {
    pub host: String,
    pub port: u16,
}

// 認証期限設定を表す構造体
pub struct AuthConfig {
    pub ttl: u64,
}
