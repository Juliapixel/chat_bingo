use actix_web::{web::{Data, Query}, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::{manager::GamesManager, Item};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature="swagger-ui", derive(utoipa::IntoParams, utoipa::ToSchema))]
pub struct GameRequest {
    /// the ULID of the game
    #[cfg_attr(feature="swagger-ui", schema(example = "01HPQVN1ENW15AWCYY8VGBKGF1"))]
    pub(super) id: Ulid
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature="swagger-ui", derive(utoipa::ToResponse, utoipa::ToSchema))]
pub struct GameData<'a> {
    #[cfg_attr(feature="swagger-ui", schema(inline))]
    items: &'a[Item],
    #[cfg_attr(feature="swagger-ui", schema(example = 7))]
    size: u32
}

#[cfg_attr(feature="swagger-ui", utoipa::path(
    get,
    path="/game/get",
    tag = "Game",
    params(GameRequest),
    responses(
        (status = 400, description = "no game with that ULID was found"),
        (status = 200, description = "the game data the client needs to start playing", body = GameData)
    )
))]
pub async fn get_game(req: HttpRequest, query: Query<GameRequest>) -> impl Responder {
    let game = req.app_data::<Data<GamesManager>>().unwrap().get_game(query.id);

    if let Some(game) = game {
        let data = GameData {
            items: game.get_items(),
            size: game.get_size()
        };

        HttpResponse::Ok().json(data)
    } else {
        HttpResponse::NotFound().body(())
    }

}
