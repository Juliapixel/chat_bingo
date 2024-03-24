use actix_web::HttpRequest;

use super::RateLimiterBackend;

/// dummy rate limiting backend, always lets requests through
#[derive(Debug, Clone, Copy, Default)]
pub struct Dummy();

impl RateLimiterBackend for Dummy {
    async fn limit(&self, _req: &HttpRequest) -> bool {
        false
    }
}

impl Dummy {
    pub fn new() -> Self {
        Self()
    }
}
