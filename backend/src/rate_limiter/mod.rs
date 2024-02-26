use std::{future::Future, net::IpAddr, pin::Pin, str::FromStr, time::Instant};

use actix_web::{dev::{Service, ServiceRequest, ServiceResponse, Transform}, HttpResponseBuilder, ResponseError};
use log::{debug, trace};
use thiserror::Error;

mod in_memory;
pub use in_memory::InMemory;

mod dummy;
pub use dummy::Dummy;

pub trait RateLimiterBackend {
    fn limit(&self, ip: IpAddr) -> bool;
}

/// Rate limiter middleware
///
/// note: this should *probably* be created outside of HttpServer::new()
#[derive(Debug, Clone, Default)]
pub struct RateLimiter<B: RateLimiterBackend>(B);

impl<B: RateLimiterBackend> RateLimiter<B> {
    pub fn new(backend: B) -> Self {
        Self(backend)
    }
}

impl<S, B, BA> Transform<S, ServiceRequest> for RateLimiter<BA>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    BA: RateLimiterBackend + Clone
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = RateLimiterMiddleware<S, BA>;
    type InitError = ();
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(RateLimiterMiddleware{service, backend: self.0.clone()}))
    }
}

pub struct RateLimiterMiddleware<S: Service<ServiceRequest>, B: RateLimiterBackend> {
    service: S,
    backend: B
}

#[derive(Debug, Clone, Copy, Error)]
#[error("too many requests")]
pub struct RateLimiterError();

impl ResponseError for RateLimiterError {
    fn status_code(&self) -> reqwest::StatusCode {
        reqwest::StatusCode::TOO_MANY_REQUESTS
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        HttpResponseBuilder::new(self.status_code()).body("too many requests")
    }
}

impl<S, B, BA> Service<ServiceRequest> for RateLimiterMiddleware<S, BA>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    BA: RateLimiterBackend
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, ctx: &mut core::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let ip = if crate::cli::ARGS.reverse_proxy_mode {
            IpAddr::from_str(req.connection_info().realip_remote_addr().unwrap()).unwrap()
        } else {
            IpAddr::from_str(req.connection_info().peer_addr().unwrap()).unwrap()
        };
        let is_limited = self.backend.limit(ip);
        let fut = self.service.call(req);
        Box::pin(async move {
            if is_limited {
                debug!("request from {ip} rate limited");
                Err(RateLimiterError().into())
            } else {
                fut.await
            }
        })
    }
}
