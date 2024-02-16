use serde::{Deserialize, Serialize};

// TODO: make this do things
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

// TODO: add client-sent events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientEvent {

}
