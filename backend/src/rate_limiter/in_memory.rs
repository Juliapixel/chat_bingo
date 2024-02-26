use std::{net::IpAddr, sync::Arc, time::{Duration, Instant}};

use dashmap::DashMap;
use parking_lot::RwLock;

use super::RateLimiterBackend;

/// in-memory global rolling window implementation for rate limiting
/// # implementation
/// should be initialized outside of HttpServer::new and cloned inside of it
#[derive(Debug, Clone)]
pub struct InMemory {
    storage: Arc<DashMap<IpAddr, u32>>,
    request_limit: u32,
    window_size: Duration,
    window_start: Arc<RwLock<Instant>>
}

impl RateLimiterBackend for InMemory {
    fn limit(&self, ip: IpAddr) -> bool {
        self.clear_if_window_passed();
        self.track_address(ip);
        if let Some(count) = self.storage.get(&ip) {
            *count > self.request_limit
        } else {
            false
        }
    }
}

impl InMemory {
    pub fn new(window_size: Duration, max_requests: u32) -> Self {
        Self {
            storage: Arc::new(DashMap::new()),
            window_size,
            request_limit: max_requests,
            window_start: Arc::new(RwLock::new(Instant::now()))
        }
    }

    fn clear_if_window_passed(&self) {
        if Instant::now() > *self.window_start.read() + self.window_size {
            self.storage.retain(|_, _| false);
            *self.window_start.write() = Instant::now();
        }
    }

    fn track_address(&self, ip: IpAddr) {
        self.clear_if_window_passed();
        if let Some(mut count) = self.storage.get_mut(&ip) {
            *count += 1;
        } else {
            self.storage.insert(ip, 1);
        }
    }
}
