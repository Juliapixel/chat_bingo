use std::sync::OnceLock;

use bingo_backend::app_info::AppInfo;
use clap::{Args, Parser};

/// returns a lazily initiated global instance of [Arguments]
pub fn args() -> &'static Arguments {
    static ARGS: OnceLock<Arguments> = OnceLock::new();

    ARGS.get_or_init(|| {
        let _ = dotenvy::dotenv();

        Arguments::parse()
    })
}

#[derive(Parser)]
pub struct Arguments {
    #[arg(short, long, default_value = "8080")]
    pub port: u16,
    /// whether X-Forwaded-For headers are trusted or not
    #[arg(long, env="REVERSE_PROXY_MODE")]
    pub reverse_proxy_mode: bool,
    #[command(flatten)]
    pub pg_args: PgArgs,
    #[command(flatten)]
    pub app_info: AppInfo
}

#[derive(Args)]
pub struct PgArgs {
    #[arg(long, env="PG_HOST", default_value = "localhost")]
    pub pg_host: String,
    #[arg(long, env="PG_PORT", default_value = "5432")]
    pub pg_port: u16,
    #[arg(long, env="PG_DB", default_value = "bingo")]
    pub database: String,
    #[arg(long, env="PG_PASSWORD")]
    pub password: String,
    #[arg(long, env="PG_USERNAME", default_value = "postgres")]
    pub username: String,
}
