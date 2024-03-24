use std::{hash::Hash, sync::Arc, time::{Duration, Instant}};

use actix_web::{FromRequest, HttpRequest};
use dashmap::DashMap;
use parking_lot::RwLock;

use super::RateLimiterBackend;

/// in-memory global rolling window implementation for rate limiting
/// # implementation
/// should be initialized outside of HttpServer::new and cloned inside of it
#[derive(Debug, Clone)]
pub struct InMemory<T: FromRequest + Hash + Eq> {
    storage: Arc<DashMap<T, u32>>,
    request_limit: u32,
    window_size: Duration,
    window_start: Arc<RwLock<Instant>>
}

impl<T: FromRequest + Hash + Eq> RateLimiterBackend for InMemory<T> {
    async fn limit(&self, req: &HttpRequest) -> bool {
        match T::extract(req).await {
            Ok(tracked) => {
                self.clear_if_window_passed();
                if let Some(count) = self.storage.get(&tracked).map(|i| *i) {
                    self.track(tracked);
                    count > self.request_limit
                } else {
                    self.track(tracked);
                    false
                }
            },
            Err(_) => true,
        }
    }
}

impl<T: FromRequest + Hash + Eq> InMemory<T> {
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

    fn track(&self, key: T) {
        self.clear_if_window_passed();
        if let Some(mut count) = self.storage.get_mut(&key) {
            *count += 1;
        } else {
            self.storage.insert(key, 1);
        }
    }
}
