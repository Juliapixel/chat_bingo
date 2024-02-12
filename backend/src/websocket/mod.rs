use std::time::{Duration, Instant};

use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web::{web::{self, Query}, HttpRequest, Responder};
use actix_web_actors::ws;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::channel;
use ulid::Ulid;

use crate::event::ClientEvent;

use self::{event_listener::EventListener, heartbeat::Heartbeat};

mod heartbeat;
mod event_listener;

const HEARTBEAT_TIME: Duration = Duration::from_secs(29);

struct BingoWs {
    last_message: Instant,
    game: Ulid,
}

impl BingoWs {
}

impl Actor for BingoWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.spawn(Heartbeat::new(HEARTBEAT_TIME, Duration::from_secs(1)));
        ctx.spawn(EventListener::new(todo!()));
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

                    },
                    Err(e) => {

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
struct WsParams {
    game: Ulid
}

pub async fn websocket(req: HttpRequest, stream: web::Payload) -> impl Responder {
    let ws = BingoWs{
        last_message: Instant::now(),
        game: Ulid::new()
    };

    let resp = ws::start(ws, &req, stream);
    info!("{resp:?}");
    resp
}
