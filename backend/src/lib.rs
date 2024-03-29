#![allow(unused_attributes)]
#![allow(dead_code)]

#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]

#![allow(clippy::needless_return)]
#![allow(async_fn_in_trait)]

pub mod event;
pub mod game;
pub mod websocket;
pub mod auth;
pub mod app_info;
pub mod user;
pub mod cli;
pub mod rate_limiter;
pub mod metrics;
pub mod utils;
#[cfg(feature = "swagger-ui")]
pub mod doc;
