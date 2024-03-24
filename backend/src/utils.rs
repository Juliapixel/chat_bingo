use std::{future::{ready, Ready}, net::{AddrParseError, IpAddr}, ops::{Deref, DerefMut}, str::FromStr};

use actix_web::{FromRequest, HttpResponseBuilder, ResponseError};
use thiserror::Error;

use crate::cli::ARGS;

/// an extractor for the IP address of a requester, which also respects our
/// reverse proxy settings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ReqIpAddr(IpAddr);

impl FromRequest for ReqIpAddr {
    type Error = ReqIpAddrError;

    type Future = Ready<Result<ReqIpAddr, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        ready({
            if ARGS.reverse_proxy_mode {
                if let Some(ip) = req.connection_info().realip_remote_addr().map(IpAddr::from_str) {
                    match ip {
                        Ok(o) => Ok(Self(o)),
                        Err(_) => Err(ReqIpAddrError::BadIp),
                    }
                } else {
                    Err(ReqIpAddrError::NoIp)
                }
            } else {
                match req.peer_addr() {
                    Some(ip) => Ok(Self(ip.ip())),
                    None => Err(ReqIpAddrError::NoIp),
                }
            }
        })
    }
}

impl Deref for ReqIpAddr {
    type Target = IpAddr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ReqIpAddr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, Copy, Error)]
pub enum ReqIpAddrError {
    #[error("the request had no IP address")]
    NoIp,
    #[error("the IP address in the header was malformed")]
    BadIp
}

impl ResponseError for ReqIpAddrError {
    fn status_code(&self) -> reqwest::StatusCode {
        reqwest::StatusCode::BAD_REQUEST
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        HttpResponseBuilder::new(self.status_code()).body(self.to_string())
    }
}

impl From<AddrParseError> for ReqIpAddrError {
    fn from(_value: AddrParseError) -> Self {
        Self::BadIp
    }
}
