use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize)]
pub struct TokenRequestForm<'a> {
    client_id: &'a str,
    client_secret: &'a str,
    code: &'a str,
    grant_type: &'a str,
    redirect_uri: &'a str,
}

impl<'a> TokenRequestForm<'a> {
    pub fn new(client_id: &'a str, client_secret: &'a str, code: &'a str, redirect_uri: &'a str) -> Self {
        Self {
            client_id,
            client_secret,
            code,
            grant_type: "authorization_code",
            redirect_uri
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenRequestResponse {
    pub access_token: String,
    pub expires_in: i64,
    pub refresh_token: String,
    pub scope: Vec<String>,
    pub token_type: String
}

#[derive(Debug, Error)]
pub enum TokenRequestError {
    #[error(transparent)]
    RequestError(#[from] reqwest::Error),
    #[error(transparent)]
    DeserializationError(#[from] serde_json::Error)
}

pub async fn request_auth_token(form: TokenRequestForm<'_>) -> Result<TokenRequestResponse, TokenRequestError> {
    let client = reqwest::Client::new();
    let res = client.post("https://id.twitch.tv/oauth2/token")
        .form(&form)
        .send().await?;
    return Ok(serde_json::from_str::<TokenRequestResponse>(&res.text().await?)?)
}
