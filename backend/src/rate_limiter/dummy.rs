use super::RateLimiterBackend;

/// dummy rate limiting backend, always lets requests through
#[derive(Debug, Clone, Copy, Default)]
pub struct Dummy();

impl RateLimiterBackend for Dummy {
    fn limit(&self, _ip: std::net::IpAddr) -> bool {
        false
    }
}

impl Dummy {
    pub fn new() -> Self {
        Self()
    }
}
