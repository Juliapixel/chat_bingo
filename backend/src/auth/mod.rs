pub mod middleware;
pub mod twitch;
pub mod jwt;
pub mod error;

use actix_web::{
    cookie::CookieBuilder,
    http::StatusCode,
    web::{Data, Query},
    Either,
    HttpRequest,
    HttpResponse,
    HttpResponseBuilder
};
pub use middleware::TwitchAuthMiddleware;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use twitch_api::twitch_oauth2::{TwitchToken, UserToken};
use ulid::Ulid;

use crate::{app_info::AppInfo, auth::jwt::{create_new_jwt, Claims}, user::User};

use self::error::TwitchAuthError;

const JULIALUXEL_TWITCH_ID: &'static str = "173685614";

/// query params twitch sends us in case the user allowed authorization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "swagger-ui", derive(utoipa::ToSchema, utoipa::IntoParams))]
pub struct TwitchAuthParamsSuccess {
    code: String,
    scope: String,
    state: Option<String>
}

/// query params twitch sends us in case the user did not authorize us
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "swagger-ui", derive(utoipa::ToSchema, utoipa::IntoParams))]
pub struct TwitchAuthParamsError {
    error: String,
    error_description: String,
    state: Option<String>
}

type TwitchParamsQuery = Either<Query<TwitchAuthParamsSuccess>, Query<TwitchAuthParamsError>>;

/// this should only ever be used by twitch, in order to send us the data necessary to acquire an user access token from them
///
/// https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#authorization-code-grant-flow
#[cfg_attr(feature = "swagger-ui", utoipa::path(
    post,
    path = "/twitch_auth",
    responses(
        (status = 200, headers(("Set-Cookie" = String, description = "the user's app-specific JWT token"))),
        (status = 403)
    )
))]
pub async fn twitch_auth(
    _req: HttpRequest,
    params: TwitchParamsQuery,
    db_pool: Data<PgPool>,
    app_info: Data<AppInfo>
) -> Result<HttpResponse, TwitchAuthError> {
    match params {
        Either::Left(success) => {
            let client = twitch_api::HelixClient::<reqwest::Client>::new();

            let mut token_builder = UserToken::builder(
                app_info.app_id.clone().into(),
                app_info.app_secret.clone().into(),
                app_info.redirect_uri.clone()
            );
            token_builder.set_csrf("juh".into());

            let token = token_builder.get_user_token(&client, "juh", &success.code).await?;

            let display_name = client.get_user_from_id(&token.user_id, &token).await?.unwrap().display_name;

            let expiration = token.expires_in();

            let mut user = User::new(Ulid::new(), token.user_id, token.login, display_name, token.access_token);

            user.upsert(&**db_pool).await.unwrap();

            let jwt = tokio::task::spawn_blocking(move || {create_new_jwt(Claims::new(user.user_id.into(), jwt::UserKind::Player, expiration))}).await.unwrap();

            let cookie = CookieBuilder::new("jwt", jwt)
                .http_only(true)
                .secure(true)
                .same_site(actix_web::cookie::SameSite::Strict)
                .expires(time::OffsetDateTime::now_utc() + expiration)
                .finish();

            return Ok(
                HttpResponseBuilder::new(StatusCode::OK)
                    .cookie(cookie).finish()
            );
        },
        Either::Right(_error) => {
            return Ok(HttpResponseBuilder::new(StatusCode::FORBIDDEN).finish());
        },
    }
}
