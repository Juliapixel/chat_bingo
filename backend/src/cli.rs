use std::path::PathBuf;

use bingo_backend::app_info::AppInfo;
use clap::{ArgAction, Args, Parser};
use once_cell::sync::Lazy;

pub static ARGS: Lazy<Arguments> = Lazy::new(|| {
    let _ = dotenvy::dotenv();
    Arguments::parse()
});

#[derive(Parser)]
pub struct Arguments {
    #[arg(short, long, default_value = "8080")]
    pub port: u16,
    /// whether X-Forwaded-For headers are trusted or not
    #[arg(long, env="REVERSE_PROXY_MODE")]
    pub reverse_proxy_mode: bool,
    /// set verbosity of logging
    ///
    /// - 0 -> INFO
    /// - 1 -> DEBUG
    /// - 2 -> TRACE
    #[arg(short, action(ArgAction::Count), env="BINGO_VERBOSITY")]
    pub verbose: u8,
    #[arg(long, env="STATIC_FILES_PATH", value_parser = validate_path)]
    pub static_files_path: PathBuf,
    #[command(flatten)]
    pub pg_args: PgArgs,
    #[command(flatten)]
    pub app_info: AppInfo
}

#[derive(Args)]
pub struct PgArgs {
    #[arg(long, env="PG_HOST")]
    pub pg_host: String,
    #[arg(long, env="PG_PORT")]
    pub pg_port: u16,
    #[arg(long, env="PG_DB", default_value = "bingo")]
    pub database: String,
    #[arg(long, env="PG_PASSWORD")]
    pub password: String,
    #[arg(long, env="PG_USERNAME", default_value = "postgres")]
    pub username: String,
}

fn validate_path(val: &str) -> Result<PathBuf, &'static str> {
    let buf = PathBuf::from(val);

    if !buf.exists() {
        return Err("path not found");
    }
    if !buf.is_dir() {
        return Err("path must be a directory");
    }
    if !buf.is_absolute() {
        return Err("path must be absolute");
    }
    Ok(buf)
}
