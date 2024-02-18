use clap::Args;

#[derive(Clone, Args)]
pub struct AppInfo {
    #[arg(long, env)]
    pub app_id: String,
    #[arg(long, env)]
    pub app_secret: String,
    #[arg(long, env)]
    pub redirect_uri: url::Url,
}

impl AppInfo {
    pub fn new(app_id: String, app_secret: String, redirect_uri: url::Url) -> Self {
        Self { app_id, app_secret, redirect_uri }
    }
}

impl std::fmt::Debug for AppInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppInfo")
            .field("app_id", &self.app_id)
            .field("app_secret", &"[REDACTED]")
            .finish()
    }
}
