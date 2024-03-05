use std::{net::Ipv4Addr, time::Duration};

use actix_web::{
    middleware::{Compress, DefaultHeaders, Logger},
    web::{self, Data}
};
use bingo_backend::{
    auth::{self, TwitchAuthMiddleware}, cli, game::{self, manager::GamesManager}, metrics::{prometheus_endpoint, Prometheus}, rate_limiter::{Dummy, InMemory, RateLimiter}, websocket
};
use env_logger::Env;
use log::{error, info};
use sqlx::ConnectOptions;

#[cfg(feature="swagger-ui")]
use {
    utoipa::OpenApi,
    utoipa_swagger_ui::SwaggerUi
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
    // Initializing logging

    env_logger::init_from_env(
        Env::new()
            .filter_or("BINGO_LOG", match cli::ARGS.verbose {
                0 => "INFO",
                1 => "DEBUG",
                _ => "TRACE"
            })
    );

    // Connecting to database

    info!("connecting to database...");

    let db_pool = match sqlx::postgres::PgPoolOptions::new()
        .max_connections(30)
        .min_connections(1)
        .connect_with(
            sqlx::postgres::PgConnectOptions::default()
                .database(&cli::ARGS.pg_args.database)
                .port(cli::ARGS.pg_args.pg_port)
                .host(&cli::ARGS.pg_args.pg_host)
                .username(&cli::ARGS.pg_args.username)
                .password(&cli::ARGS.pg_args.password)
                .log_slow_statements(log::LevelFilter::Warn, Duration::from_millis(300))
    ).await {
        Ok(pool) => Data::new(pool),
        Err(e) => {
            error!("failed to connect to database: {e}");
            return;
        }
    };

    // Applying database migrations

    info!("applying db migrations...");

    let res = sqlx::migrate!("./migrations")
        .run(&mut db_pool.acquire().await.unwrap()).await;

    if let Err(migrate_error) = res {
        error!("{migrate_error:?}");
        return
    }

    // Initializing server shared data

    #[cfg(feature="swagger-ui")]
    let api_doc = {
        let docs = bingo_backend::doc::ApiDoc::openapi();
        info!("Swagger UI API docs availablet at /swagger-ui/#");
        docs
    };

    let manager = Data::new(GamesManager::new());

    let app_info = Data::new(cli::ARGS.app_info.clone());

    let logger_format = match cli::ARGS.reverse_proxy_mode {
        true => "%ra | %r | status: %s | took %Dms",
        false => "%a | %r | status: %s | took %Dms",
    };

    let rate_limiter = RateLimiter::new({
        #[cfg(debug_assertions)]
        {
            info!("initializing rate limiting with dummy limiter");
            Dummy::new()
        }
        #[cfg(not(debug_assertions))]
        {
            info!("initializing rate limiting with in-memory rate limiter");
            InMemory::new(Duration::from_secs(10), 100)
        }
    });

    actix_web::HttpServer::new(move || {
        let app = actix_web::App::new()
            .app_data(app_info.clone())
            .app_data(manager.clone())
            .app_data(db_pool.clone())
            .wrap(Prometheus::new())
            .wrap(rate_limiter.clone())
            .wrap(Logger::new(logger_format))
            .wrap(TwitchAuthMiddleware::default())
            .wrap(
                DefaultHeaders::new()
                    .add(("Access-Control-Allow-Origin", "*"))
                    .add(("Access-Control-Allow-Methods", "*"))
                    .add(("Access-Control-Allow-Headers", "*"))
            )
            .wrap(Compress::default())
            .service(web::resource("/metrics").get(prometheus_endpoint))
            .service(web::resource("/ws").get(websocket::websocket))
            .service(web::resource("/twitch_auth").get(auth::twitch_auth))
            .service(web::scope("/game").configure(game::configure));

        #[cfg(feature="swagger-ui")]
        let app = app.service(
            SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", api_doc.clone())
        );

        app
    }).bind((BIND_ADDRESS, cli::ARGS.port))
    .unwrap_or_else(|e| panic!("unable to bind server to {BIND_ADDRESS}:{}: {e}", cli::ARGS.port))
    .run().await.unwrap();
}
