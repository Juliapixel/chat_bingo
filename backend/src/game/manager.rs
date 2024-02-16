use std::sync::Arc;

use hashbrown::HashMap;
use parking_lot::RwLock;
use ulid::Ulid;

use super::Game;

#[derive(Debug, Default)]
pub struct GamesManager {
    games: RwLock<HashMap<Ulid, Arc<Game>>>
}

impl GamesManager {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn new_game(&self, game: Game) {
        self.games.write().insert(game.id, Arc::new(game));
    }

    pub fn remove_game(&self, id: Ulid) {
        self.games.write().remove(&id);
    }

    pub fn get_game(&self, id: Ulid) -> Option<Arc<Game>> {
        self.games.read().get(&id).map(|a| Arc::clone(a))
    }
}
