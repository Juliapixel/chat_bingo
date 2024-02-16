use rand::Rng;

#[derive(Debug, Clone)]
pub struct PlayerData {
    /// indices of items
    board: Box<[usize]>
}

impl PlayerData {
    pub fn new_random(board_size: u32, items_len: usize) -> Self {
        let mut rng = rand::thread_rng();
        let mut items = Vec::with_capacity(items_len);
        for i in 0..items.capacity() {
            items.push(i)
        }
        for i in 0..items.len() {
            let random = rng.gen_range(0..(items.len() - 1));
            items.swap(random, items_len - 1 - i)
        }

        let mut board = Vec::with_capacity((board_size * board_size) as usize);

        for _ in 0..board.capacity() {
            board.push(rng.gen_range(0..items_len));
        }

        Self {
            board: board.into()
        }
    }
}
