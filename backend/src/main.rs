use std::time::Duration;

use actix_web::{middleware::{Compress, DefaultHeaders, Logger}, web::{self, Data}};
use bingo_backend::{auth::TwitchAuth, game::{self, manager::GamesManager}, websocket};
use env_logger::Env;
use log::{error, info};
use sqlx::ConnectOptions;

#[cfg(feature="swagger-ui")]
use {
    utoipa::OpenApi,
    utoipa_swagger_ui::SwaggerUi
};

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

    let manager = Data::new(GamesManager::new());

    info!("connecting to database...");

    // FIXME: remove expect()s
    let db_pool = Data::new(sqlx::PgPool::connect_with(
        sqlx::postgres::PgConnectOptions::default()
            .database("bingo")
            .port(dotenvy::var("POSTGRES_PORT").unwrap_or("5432".into()).parse().expect("port should be a number dumbass"))
            .host(&dotenvy::var("POSTGRES_HOST").unwrap_or("localhost".into()))
            .username("postgres")
            .password(&dotenvy::var("POSTGRES_PASSWORD").unwrap())
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
    }).bind(("127.0.0.1", 8080)).unwrap().run().await.unwrap();
}
