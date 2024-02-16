use std::time::{Duration, Instant};

use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web::{http::header::TryIntoHeaderValue, web::{self, Data, Query}, Error, HttpRequest, HttpResponse, HttpResponseBuilder, Responder, ResponseError};
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
}

impl BingoWs {
}

impl Actor for BingoWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.spawn(Heartbeat::new(HEARTBEAT_TIME, Duration::from_secs(1)));
        ctx.spawn(EventListener::new(self.listener.take().unwrap()));
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

#[cfg_attr(feature = "swagger-ui", utoipa::path(
    get,
    path = "/ws",
    params(WsParams),
    responses(
        (status = 101, description = "Upgrading connection to a websocket"),
        (status = 400, description = "Game wasn't found", body = WsRequestError)
    )
))]
pub async fn websocket(req: HttpRequest, stream: web::Payload, params: Query<WsParams>) -> impl Responder {
    let distributer = req.app_data::<Data<GamesManager>>().unwrap();
    if let Some(rx) = distributer.get_game(params.game).map(|g| g.subscribe_to()) {
        let ws = BingoWs{
            last_message: Instant::now(),
            game: params.game,
            listener: Some(rx),
        };
        let resp = ws::start(ws, &req, stream);
        info!("{resp:?}");
        resp
    } else {
        Err(Error::from(WsRequestError::NoSuchGame))
    }
}
