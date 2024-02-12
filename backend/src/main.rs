use std::sync::Arc;

use actix_web::{middleware::Logger, web};
use env_logger::Env;
use hashbrown::HashMap;
use log::info;
use parking_lot::RwLock;
use tokio::sync::broadcast::Receiver;
use ulid::Ulid;

use crate::event::ServerEvent;

mod websocket;
mod event;

const DEFAULT_LEVEL: &'static str = {
    #[cfg(debug_assertions)]
    {
        "DEBUG"
    }
    #[cfg(not(debug_assertions))]
    {
        "INFO"
    }
};

#[tokio::main]
async fn main() {
    env_logger::init_from_env(
        Env::new()
            .filter_or("BINGO_LOG", DEFAULT_LEVEL)
    );

    info!("Chat Bingo initiated pag");


    actix_web::HttpServer::new(move || {
        let events: HashMap<Ulid, Receiver<ServerEvent>> = HashMap::new();

        actix_web::App::new()
            .app_data(Arc::new(RwLock::new(events)))
            .wrap(Logger::default())
            .service(web::resource("ws").to(websocket::websocket))
    }).bind(("127.0.0.1", 8080)).unwrap().run().await.unwrap();
}
