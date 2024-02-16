use std::sync::Arc;

use actix_web::web;
use hashbrown::HashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::{Receiver, Sender};
use ulid::Ulid;

use crate::event::ServerEvent;

use self::playerdata::PlayerData;

pub mod manager;
pub mod playerdata;
pub mod create;
pub mod get;
pub mod update;

#[cfg(test)]
mod test;

#[derive(Debug)]
pub struct Game {
    id: Ulid,
    size: u32,
    items: Box<[Item]>,
    players: RwLock<HashMap<Ulid, PlayerData>>,
    event_sender: Sender<ServerEvent>,
}

impl Game {
    pub fn new(id: Ulid, size: u32, items: Box<[Item]>) -> Self {
        assert!(items.len() >= (size * size) as usize, "there must be at least {} items in a board of size {}, but there were only {}", size.pow(2), size, items.len());
        let (tx, _rx) = tokio::sync::broadcast::channel(8);
        Self {
            id,
            size,
            items,
            players: Default::default(),
            event_sender: tx
        }
    }

    pub fn get_items(&self) -> &[Item] {
        &self.items
    }

    pub fn get_size(&self) -> u32 {
        self.size
    }

    pub fn subscribe_to(&self) -> Receiver<ServerEvent> {
        self.event_sender.subscribe()
    }

    pub fn send_event(&self, event: ServerEvent) {
        self.event_sender.send(event);
    }

    pub fn add_new_player(&self, id: Ulid) {
        self.players.write().insert(id, PlayerData::new_random(self.size, self.items.len()));
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature="swagger-ui", derive(utoipa::ToSchema))]
pub struct Item {
    text: Arc<str>,
    picked: bool,
}

impl From<String> for Item {
    fn from(value: String) -> Self {
        Self {
            text: value.into(),
            picked: false
        }
    }
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/get").get(get::get_game))
        .service(web::resource("/create").post(create::create_game));
}
