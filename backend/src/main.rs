use std::time::Duration;

use actix_web::{middleware::{Compress, DefaultHeaders, Logger}, web::{self, Data}};
use bingo_backend::{auth::TwitchAuth, game::{self, manager::GamesManager}, websocket};
use clap::Parser;
use env_logger::Env;
use log::{error, info};
use sqlx::ConnectOptions;

use crate::cli::Arguments;

#[cfg(feature="swagger-ui")]
use {
    utoipa::OpenApi,
    utoipa_swagger_ui::SwaggerUi
};

mod cli;

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
    let _ = dotenvy::dotenv();

    let args = Arguments::parse();

    env_logger::init_from_env(
        Env::new()
            .filter_or("BINGO_LOG", DEFAULT_LEVEL)
    );

    let manager = Data::new(GamesManager::new());

    info!("connecting to database...");

    let db_pool = Data::new(sqlx::PgPool::connect_with(
        sqlx::postgres::PgConnectOptions::default()
            .database(&args.pg_args.database)
            .port(args.pg_args.pg_port)
            .host(&args.pg_args.pg_host)
            .username(&args.pg_args.username)
            .password(&args.pg_args.password)
            .log_slow_statements(log::LevelFilter::Warn, Duration::from_millis(300))
    ).await.expect("failed to connect to db erm"));

    info!("applying db migrations...");

    let res = sqlx::migrate!("./migrations")
        .run(&mut db_pool.acquire().await.unwrap()).await;

    if let Err(migrate_error) = res {
        error!("{migrate_error:?}");
        return
    }

    #[cfg(feature="swagger-ui")]
    let api_doc = {
        let docs = bingo_backend::doc::ApiDoc::openapi();
        info!("Swagger UI API docs availablet at /swagger-ui/#");
        docs
    };

    actix_web::HttpServer::new(move || {
        let app = actix_web::App::new()
            .app_data(manager.clone())
            .app_data(db_pool.clone())
            .wrap(Compress::default())
            .wrap(TwitchAuth::default())
            .wrap(Logger::default())
            .wrap(DefaultHeaders::new().add(("Server", "actix-web")))
            .service(web::resource("/ws").to(websocket::websocket))
            .service(web::scope("/game").configure(game::configure));

        #[cfg(feature="swagger-ui")]
        let app = app.service(
            SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", api_doc.clone())
        );

        return app
    }).bind(("127.0.0.1", args.port)).unwrap().run().await.unwrap();
}
