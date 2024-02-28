use actix_web::{HttpResponseBuilder, Responder};
use prometheus::TextEncoder;
use reqwest::StatusCode;

mod middleware;
pub use middleware::Prometheus;

/// FIXME: limit this so only prometheus can access it
pub async fn prometheus_endpoint() -> impl Responder {
    match TextEncoder::new().encode_to_string(&prometheus::gather()) {
        Ok(encoded) => HttpResponseBuilder::new(StatusCode::OK).body(encoded),
        Err(_) => HttpResponseBuilder::new(StatusCode::INTERNAL_SERVER_ERROR).finish()
    }
}
