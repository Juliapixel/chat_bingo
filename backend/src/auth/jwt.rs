use std::{sync::OnceLock, time::Duration};

use base64::Engine;
use jsonwebtoken::{EncodingKey, Header};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

fn get_jwt_secret() -> &'static [u8] {
    static SECRET: OnceLock<Vec<u8>> = OnceLock::new();

    SECRET.get_or_init(|| {
        #[cfg(debug_assertions)]
        {
            base64::prelude::BASE64_STANDARD.decode(dotenvy::var("JWT_SECRET").unwrap()).unwrap()
        }
        #[cfg(not(debug_assertions))]
        {
            todo!("HELPPPP")
        }
    })
}

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
    jsonwebtoken::encode(&Header::default(), &claims, &EncodingKey::from_secret(get_jwt_secret())).unwrap()
}
