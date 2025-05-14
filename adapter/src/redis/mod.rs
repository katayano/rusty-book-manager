pub mod model;

use redis::{AsyncCommands, Client};
use shared::{config::RedisConfig, error::AppResult};

use self::model::{RedisKey, RedisValue};

pub struct RedisClient {
    client: Client,
}

impl RedisClient {
    /// Redis接続用のクライアントを初期化する
    pub fn new(config: &RedisConfig) -> AppResult<Self> {
        let client = Client::open(format!("redis://{}:{}", config.host, config.port))?;
        Ok(Self { client })
    }
    /// 期限付きでKey, Valueを保存する。指定した期限が過ぎるとRedis上から該当データが消える
    pub async fn set_ex<T: RedisKey>(&self, key: &T, value: &T::Value, ttl: u64) -> AppResult<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let _: () = conn.set_ex(key.inner(), value.inner(), ttl).await?;
        Ok(())
    }
    /// Keyを指定して、Valueを取得する
    pub async fn get<T: RedisKey>(&self, key: &T) -> AppResult<Option<T::Value>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let value: Option<String> = conn.get(key.inner()).await?;
        value.map(T::Value::try_from).transpose()
    }
    /// Keyを指定して、Redis上の該当のKeyとValueを削除する
    pub async fn delete<T: RedisKey>(&self, key: &T) -> AppResult<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let _: () = conn.del(key.inner()).await?;
        Ok(())
    }
    /// 接続を確認する。ヘルスチェック用
    pub async fn try_connect(&self) -> AppResult<()> {
        let _ = self.client.get_multiplexed_async_connection().await?;
        Ok(())
    }
}
