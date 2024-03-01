use std::time::{Duration, Instant};

use actix::{Actor, ActorContext, AsyncContext, SpawnHandle, StreamHandler};
use actix_web::{web::{self, Data, Query}, Error, HttpRequest, HttpResponse, HttpResponseBuilder, Responder, ResponseError};
use actix_web_actors::ws;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::Receiver;
use ulid::Ulid;

use crate::{event::{ClientEvent, ServerEvent}, game::manager::GamesManager};

use self::{event_listener::EventListener, heartbeat::Heartbeat};

mod heartbeat;
mod event_listener;

const HEARTBEAT_TIME: Duration = Duration::from_secs(29);

struct BingoWs {
    last_message: Instant,
    game: Ulid,
    listener: Option<Receiver<ServerEvent>>,
    heartbeat_handle: Option<SpawnHandle>,
    listener_handle: Option<SpawnHandle>
}

impl BingoWs {
    pub(self) fn new(receiver: Receiver<ServerEvent>, game_ulid: Ulid) -> Self {
        Self {
            last_message: Instant::now(),
            game: game_ulid,
            listener: Some(receiver),
            heartbeat_handle: None,
            listener_handle: None
        }
    }
}

impl Actor for BingoWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.heartbeat_handle = Some(ctx.spawn(Heartbeat::new(HEARTBEAT_TIME, Duration::from_secs(1))));
        self.listener_handle = Some(ctx.spawn(EventListener::new(self.listener.take().unwrap())));
    }

    // these are both cancel-safe since no important data is stored in them
    fn stopped(&mut self, ctx: &mut Self::Context) {
        if let Some(h) = self.heartbeat_handle {
            ctx.cancel_future(h);
        }
        if let Some(h) = self.listener_handle {
            ctx.cancel_future(h);
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for BingoWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        self.last_message = Instant::now();

        debug!("received {msg:?}");

        match msg {
            Ok(ws::Message::Text(text)) => {
                match serde_json::from_str::<ClientEvent>(&text) {
                    Ok(o) => {
                        todo!("handle received WS messages")
                    },
                    Err(e) => {
                        todo!("implement handling for deserializing received WS messages")
                    },
                }
            },
            Ok(ws::Message::Close(_)) => ctx.stop(),
            Ok(_) => (),
            Err(e) => error!("{e}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "swagger-ui", derive(utoipa::IntoParams))]
pub struct WsParams {
    /// the ULID of the game the client wishes to monitor
    game: Ulid
}

#[derive(Debug, Clone, Serialize, thiserror::Error)]
#[serde(rename_all = "snake_case", tag = "error")]
#[cfg_attr(feature = "swagger-ui", derive(utoipa::ToSchema))]
pub enum WsRequestError {
    #[error("there is no such game")]
    NoSuchGame,
}

impl ResponseError for WsRequestError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::BAD_REQUEST
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        HttpResponseBuilder::new(self.status_code()).json(self)
    }
}

/// websocket connection to give live game updates to players
#[cfg_attr(feature = "swagger-ui", utoipa::path(
    get,
    path = "/ws",
    tag = "Game",
    params(WsParams),
    responses(
        (status = 101, description = "Upgrading connection to a websocket"),
        (status = 400, description = "Game wasn't found", body = WsRequestError)
    )
))]
pub async fn websocket(
    req: HttpRequest,
    stream: web::Payload,
    params: Query<WsParams>,
    games_manager: Data<GamesManager>
) -> impl Responder {
    if let Some(rx) = games_manager.get_game(params.game).map(|g| g.subscribe_to()) {
        let ws = BingoWs::new(rx, params.game);
        let resp = ws::start(ws, &req, stream);
        info!("{resp:?}");
        resp
    } else {
        Err(Error::from(WsRequestError::NoSuchGame))
    }
}
