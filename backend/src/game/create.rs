use actix_web::{web::{Data, Json}, HttpRequest, HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::{manager::GamesManager, Game, Item};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature="swagger-ui", derive(utoipa::ToSchema))]
pub struct CreateGameRequest {
    // name: String,
    pub(super) items: Vec<String>,
    #[cfg_attr(feature="swagger-ui", schema(minimum = 5, maximum = 23))]
    pub(super) size: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, thiserror::Error)]
#[serde(rename_all="snake_case", tag="error")]
#[cfg_attr(feature="swagger-ui", derive(utoipa::ToSchema))]
pub enum CreateError {
    #[error("there were not enough items to fill a bingo card")]
    NotEnoughItems,
    #[error("the size of the requested bingo board was not odd")]
    SizeNotOdd,
    #[error("the board was too big!")]
    TooBig,
    #[error("the board was too small")]
    TooSmall
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature="swagger-ui", derive(utoipa::ToResponse, utoipa::ToSchema))]
pub struct CreatedGame {
    #[cfg_attr(feature="swagger-ui", schema(example = "01HPQVN1ENW15AWCYY8VGBKGF1"))]
    pub(super) id: Ulid
}

impl ResponseError for CreateError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::BAD_REQUEST
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let resp = HttpResponse::new(self.status_code())
            .set_body(serde_json::to_string(self).unwrap())
            .map_into_boxed_body();

        return resp;
    }
}

#[cfg_attr(feature="swagger-ui", utoipa::path(
    post,
    path = "/game/create",
    security(
        ("user_token" = ["write:games"])
    ),
    request_body(
        content = CreateGameRequest,
        description = "The information of a basic bingo name, `size` must be an
        odd number between 5 and 23 inclusive, and `items` must be of minimum length `size`^2"
    ),
    responses(
        (status = 200, description = "A game was created succesfully", body = CreatedGame),
        (status = 400, description = "The request to create a game was invalid", body = CreateError)
    )
))]
pub async fn create_game(req: HttpRequest, game: Json<CreateGameRequest>) -> Result<Json<CreatedGame>, CreateError> {
    match game.size {
        x if x > 23  => return Err(CreateError::TooBig),
        x if x < 5 => return Err(CreateError::TooSmall),
        x if x % 2 != 1 => return Err(CreateError::SizeNotOdd),
        x if game.items.len() < x.pow(2) as usize => return Err(CreateError::NotEnoughItems),
        _ => ()
    };
    let games = req.app_data::<Data<GamesManager>>().unwrap();

    let ulid = Ulid::new();

    let items: Box<[Item]> = game.0.items.into_iter().map(|i| i.into()).collect();

    games.new_game(Game::new(ulid, game.0.size, items));

    return Ok(Json(CreatedGame { id: ulid }))
}
