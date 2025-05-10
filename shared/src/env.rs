use std::env;
use strum::EnumString;

#[derive(Default, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum Environment {
    #[default]
    Development, // 開発環境
    Production, // 本番環境
}

/// 開発環境・本番環境のどちら向けのビルドかを示す
pub fn which() -> Environment {
    // debug_assertionsがonの場合は開発環境
    // そうでない場合は本番環境と判定
    // 以下のlet defualt_envの行は片方だけ実行される
    #[cfg(debug_assertions)]
    let default_env = Environment::Development;
    #[cfg(not(debug_assertions))]
    let default_env = Environment::Production;

    match env::var("ENV") {
        Ok(env) => env.parse().unwrap_or(default_env),
        Err(_) => default_env,
    }
}
