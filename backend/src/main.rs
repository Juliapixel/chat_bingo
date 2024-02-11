use actix_web::{middleware::Logger, web};
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

#[tokio::main]
async fn main() {
    env_logger::init_from_env(
        Env::new()
            .filter_or("BINGO_LOG", DEFAULT_LEVEL)
    );

    info!("Chat Bingo initiated pag");

    actix_web::HttpServer::new(|| {
        actix_web::App::new()
            .wrap(Logger::default())
            .service(web::resource("/").to(|| async { "Hello, world!" }))
    }).bind(("127.0.0.1", 8080)).unwrap().run().await.unwrap();
}
