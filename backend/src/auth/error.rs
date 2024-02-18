use actix_web::ResponseError;
use log::error;
use reqwest::StatusCode;
use serde::Serialize;
use thiserror::Error;
use twitch_api::{helix::ClientRequestError, twitch_oauth2::tokens::errors::UserTokenExchangeError};

#[derive(Debug, Serialize, Error)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum TwitchAuthError {
    #[error("the user denied our authorization request")]
    AuthorizationDenied,
    #[error("twitch returned an invalid response: {0}")]
    #[serde(untagged)]
    BadResponseFromTwitch(&'static str),
}

impl ResponseError for TwitchAuthError {
    fn status_code(&self) -> StatusCode {
        match self {
            TwitchAuthError::BadResponseFromTwitch(_) => StatusCode::BAD_GATEWAY,
            TwitchAuthError::AuthorizationDenied => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        actix_web::HttpResponseBuilder::new(self.status_code()).json(self)
    }
}

impl<T: std::error::Error + Sync + Send> From<UserTokenExchangeError<T>> for TwitchAuthError {
    fn from(value: UserTokenExchangeError<T>) -> Self {
        error!("failed at requesting token from twitch: {value:?}");
        TwitchAuthError::BadResponseFromTwitch("failed at requesting token from twitch")
    }
}

impl<T: std::error::Error + Sync + Send> From<ClientRequestError<T>> for TwitchAuthError {
    fn from(value: ClientRequestError<T>) -> Self {
        error!("failed requesting display name from twitch: {value:?}");
        TwitchAuthError::BadResponseFromTwitch("failed requesting display name from twitch")
    }
}
