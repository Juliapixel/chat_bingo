use crate::app_info::AppInfo;
use chrono::DateTime;
use clap::{crate_version, ArgAction, Args, Parser};
use once_cell::sync::Lazy;

pub static ARGS: Lazy<Arguments> = Lazy::new(|| {
    let _ = dotenvy::dotenv();
    Arguments::parse()
});

static HELP_FORMAT: Lazy<String> = Lazy::new(|| {
    format!("Chat Bingo Backend v{}
{{about-with-newline}}(C) {{author}}
{{before-help}}
{{usage-heading}} {{usage}}

{{all-args}}{{after-help}}", crate_version!())
});

fn truncate(val: &str, len: usize) -> &str {
    let mut end_idx = 0;
    let mut chars = val.char_indices();
    for _ in 0..len {
        if let Some((idx, c)) = chars.next() {
            end_idx = idx + c.len_utf8();
        } else {
            break;
        }
    }
    return &val[0..end_idx];
}

static VERSION: Lazy<String> = Lazy::new(|| {
    format!(
        "v{}\ncommit: {}\nbranch: {}\nbuilt at: {}",
        crate_version!(),
        truncate(env!("VERGEN_GIT_SHA"), 8),
        env!("VERGEN_GIT_BRANCH"),
        DateTime::parse_from_rfc3339(env!("VERGEN_BUILD_TIMESTAMP"))
            .map(|dt| dt.with_timezone(&chrono::Local).format("%F %T %z").to_string())
            .unwrap_or("unknown date".to_string())
    )
});

/// The backend service for Chat Bingo
#[derive(Parser)]
#[command(version = &**VERSION, author, help_template = &**HELP_FORMAT)]
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
