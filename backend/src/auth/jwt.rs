use std::{future::ready, time::Duration};

use actix_web::{FromRequest, ResponseError};
use base64::Engine;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ulid::Ulid;

static JWT_SECRET: Lazy<(EncodingKey, DecodingKey)> = Lazy::new(|| {
    #[cfg(debug_assertions)]
    {
        let secret = base64::prelude::BASE64_STANDARD.decode(dotenvy::var("JWT_SECRET").unwrap()).unwrap();
        (
            EncodingKey::from_secret(&secret),
            DecodingKey::from_secret(&secret)
        )
    }
    #[cfg(not(debug_assertions))]
    {
        todo!("HELPPPP")
    }
});

static JWT_VALIDATION: Lazy<Validation> = Lazy::new(|| { Validation::new(Algorithm::default()) });

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Issued At
    ///
    /// milliseconds since the unix epoch
    iat: i64,
    /// Expiration
    ///
    /// milliseconds since the unix epoch
    exp: i64,
    user_id: Ulid,
    #[serde(skip_serializing_if = "UserKind::is_user", default)]
    user_kind: UserKind
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserKind {
    #[default]
    Player,
    Host,
    Admin
}

impl UserKind {
    pub fn is_user(&self) -> bool {
        *self == Self::Player
    }
}

impl Claims {
    pub fn new(user_id: Ulid, user_kind: UserKind, expires_in: Duration) -> Self {
        let now = chrono::Utc::now();
        Self {
            iat: now.timestamp_millis(),
            exp: (now + expires_in).timestamp_millis(),
            user_id,
            user_kind
        }
    }
}

pub fn create_new_jwt(claims: Claims) -> String {
    jsonwebtoken::encode(&Header::default(), &claims, &JWT_SECRET.0)
        .expect("failed to encode JWT token")
}

pub fn validate_jwt(token: &str) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error>{
    jsonwebtoken::decode(token, &JWT_SECRET.1, &JWT_VALIDATION)
}

#[derive(Debug, Clone, Error)]
pub enum ClaimsExtractorError {
    #[error("there was no jwt cookie in the request")]
    NoCookie,
    #[error("the provided cookie was missing")]
    InvalidToken(#[from] jsonwebtoken::errors::Error)
}

impl ResponseError for ClaimsExtractorError {
    fn status_code(&self) -> reqwest::StatusCode {
        reqwest::StatusCode::UNAUTHORIZED
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        actix_web::HttpResponseBuilder::new(self.status_code())
            .body(self.to_string())
    }
}

impl FromRequest for Claims {
    type Error = ClaimsExtractorError;

    type Future = std::future::Ready<Result<Claims, ClaimsExtractorError>>;

    fn from_request(req: &actix_web::HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        match req.cookie("jwt") {
            Some(s) => {
                ready(validate_jwt(s.value()).map(|i| i.claims).map_err(|e| e.into()))
            },
            None => ready(Err(ClaimsExtractorError::NoCookie)),
        }
    }
}
