use utoipa::{Modify, OpenApi};
use crate::{game::{create, get, update}, websocket, auth};


#[derive(OpenApi)]
#[openapi(
    modifiers(
        &AddUserAuth
    ),
    paths(
        create::create_game,
        get::get_game,
        websocket::websocket,
        auth::twitch_auth
    ),
    components(
        schemas(
            crate::game::Item,
            get::GameRequest, get::GameData,
            create::CreateGameRequest, create::CreatedGame, create::CreateError,
            websocket::WsRequestError
        ),
        responses(create::CreatedGame, get::GameData)
    )
)]
pub struct ApiDoc;

struct AddUserAuth;

impl Modify for AddUserAuth {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi.components.as_mut().map(|c| {
            c.security_schemes.insert(
                "user_token".into(),
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .description(Some("the token users get after logging in with twitch"))
                        .build()
                )
            )
        });
    }
}
