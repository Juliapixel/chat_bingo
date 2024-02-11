use env_logger::Env;
use log::info;

fn main() {
    env_logger::init_from_env(
        Env::new()
            .filter_or("BINGO_LOG", "INFO")
    );

    info!("Chat Bingo initiated pag");
}
