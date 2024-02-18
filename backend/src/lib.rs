#![allow(unused_attributes)]
#![allow(dead_code)]

#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![allow(clippy::needless_return)]

pub mod event;
pub mod game;
pub mod websocket;
pub mod auth;
pub mod app_info;
pub mod user;
#[cfg(feature = "swagger-ui")]
pub mod doc;
