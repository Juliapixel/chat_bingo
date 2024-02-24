use std::{net::Ipv4Addr, time::Duration};

use actix_web::{middleware::{Compress, DefaultHeaders, Logger}, web::{self, Data}};
use bingo_backend::{auth::{self, TwitchAuthMiddleware}, game::{self, manager::GamesManager}, websocket};
use env_logger::Env;
use log::{error, info};
use sqlx::ConnectOptions;

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

const BIND_ADDRESS: Ipv4Addr = {
    #[cfg(debug_assertions)]
    {
        Ipv4Addr::LOCALHOST
    }
    #[cfg(not(debug_assertions))]
    {
        Ipv4Addr::UNSPECIFIED
    }
};

#[tokio::main]
async fn main() {
    let args = cli::args();

    env_logger::init_from_env(
        Env::new()
            .filter_or("BINGO_LOG", DEFAULT_LEVEL)
    );

    let manager = Data::new(GamesManager::new());

    info!("connecting to database...");

    let db_pool = match sqlx::postgres::PgPoolOptions::new()
        .max_connections(30)
        .min_connections(1)
        .connect_with(
            sqlx::postgres::PgConnectOptions::default()
                .database(&args.pg_args.database)
                .port(args.pg_args.pg_port)
                .host(&args.pg_args.pg_host)
                .username(&args.pg_args.username)
                .password(&args.pg_args.password)
                .log_slow_statements(log::LevelFilter::Warn, Duration::from_millis(300))
    ).await {
        Ok(pool) => Data::new(pool),
        Err(e) => {
            error!("failed to connect to database: {e}");
            return;
        }
    };

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

    let app_info = Data::new(args.app_info.clone());

    let logger_format = match args.reverse_proxy_mode {
        true => "%ra | %r | %s",
        false => "%a | %r | %s",
    };

    actix_web::HttpServer::new(move || {
        let app = actix_web::App::new()
            .app_data(app_info.clone())
            .app_data(manager.clone())
            .app_data(db_pool.clone())
            .wrap(Compress::default())
            .wrap(TwitchAuthMiddleware::default())
            .wrap(Logger::new(logger_format))
            .wrap(DefaultHeaders::new().add(("Server", "actix-web")))
            .service(web::resource("/ws").get(websocket::websocket))
            .service(web::resource("/twitch_auth").get(auth::twitch_auth))
            .service(web::scope("/game").configure(game::configure));

        #[cfg(feature="swagger-ui")]
        let app = app.service(
            SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", api_doc.clone())
        );

        return app
    }).bind((BIND_ADDRESS, args.port))
    .unwrap_or_else(|e| panic!("{}: {e}", format!("unable to bind server to {}:{}", BIND_ADDRESS, args.port)))
    .run().await.unwrap();
}
