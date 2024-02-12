use hashbrown::HashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::{Receiver, Sender};
use ulid::Ulid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerEvent {
    NewBall {
        idx: usize,
    },
    GameOver,
    Reconnect {
        delay: u32
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientEvent {

}

pub struct WebsocketTxRxDistributer {
    listeners: RwLock<HashMap<Ulid, Receiver<ServerEvent>>>,
    senders: RwLock<HashMap<Ulid, Sender<ClientEvent>>>
}

impl WebsocketTxRxDistributer {
    pub fn new() -> Self {
        Self {
            listeners: Default::default(),
            senders: Default::default()
        }
    }

    pub fn get_listener(&self, ulid: Ulid) -> Option<Receiver<ServerEvent>> {
        self.listeners.read().get(&ulid).map(|l| l.resubscribe())
    }

    pub fn add_listener(&self, ulid: Ulid, rx: Receiver<ServerEvent>) {
        self.listeners.write().insert(ulid, rx);
    }


    pub fn get_sender(&self, ulid: Ulid) -> Option<Sender<ClientEvent>> {
        self.senders.read().get(&ulid).map(|l| l.clone())
    }

    pub fn add_sender(&self, ulid: Ulid, tx: Sender<ClientEvent>) {
        self.senders.write().insert(ulid, tx);
    }
}
