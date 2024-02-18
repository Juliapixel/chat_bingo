#![allow(unused_attributes)]
#![allow(dead_code)]

pub mod event;
pub mod game;
pub mod websocket;
pub mod auth;
pub mod app_info;
pub mod user;
#[cfg(feature = "swagger-ui")]
pub mod doc;
