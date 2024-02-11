use env_logger::Env;
use log::info;

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

fn main() {
    env_logger::init_from_env(
        Env::new()
            .filter_or("BINGO_LOG", DEFAULT_LEVEL)
    );

    info!("Chat Bingo initiated pag");
}
