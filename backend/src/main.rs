use std::{net::Ipv4Addr, path::PathBuf, time::Duration};

use actix_files::{Files, NamedFile};
use actix_web::{dev::{fn_service, ServiceRequest, ServiceResponse}, middleware::{Compress, DefaultHeaders, Logger}, web::{self, Data}};
use bingo_backend::{auth::{self, TwitchAuthMiddleware}, game::{self, manager::GamesManager}, websocket};
use env_logger::Env;
use log::{error, info};
use once_cell::sync::Lazy;
use sqlx::ConnectOptions;

#[cfg(feature="swagger-ui")]
use {
    utoipa::OpenApi,
    utoipa_swagger_ui::SwaggerUi
};

mod cli;

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
    env_logger::init_from_env(
        Env::new()
            .filter_or("BINGO_LOG", match cli::ARGS.verbose {
                0 => "INFO",
                1 => "DEBUG",
                _ => "TRACE"
            })
    );

    let manager = Data::new(GamesManager::new());

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

    let app_info = Data::new(cli::ARGS.app_info.clone());

    let logger_format = match cli::ARGS.reverse_proxy_mode {
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
            .wrap(DefaultHeaders::new().add(("Server", "actix-web")))
            .wrap(Logger::new(logger_format))
            .service(web::resource("/ws").get(websocket::websocket))
            .service(web::resource("/twitch_auth").get(auth::twitch_auth))
            .service(web::scope("/game").configure(game::configure))
            .service(
                Files::new("/", cli::ARGS.static_files_path.clone())
                    .index_file("index.html")
                    .disable_content_disposition()
                    .default_handler(fn_service(|req: ServiceRequest| async {
                        static DEFAULT_PATH: Lazy<PathBuf> = Lazy::new(|| {
                            let mut path = cli::ARGS.static_files_path.clone();
                            path.push("index.html");
                            path
                        });
                        let file = NamedFile::open_async(&*DEFAULT_PATH).await?;
                        let res = file.into_response(req.parts().0);
                        Ok(ServiceResponse::new(req.into_parts().0, res))
                    }))
            );

        #[cfg(feature="swagger-ui")]
        let app = app.service(
            SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", api_doc.clone())
        );

        app
    }).bind((BIND_ADDRESS, cli::ARGS.port))
    .unwrap_or_else(|e| panic!("unable to bind server to {BIND_ADDRESS}:{}: {e}", cli::ARGS.port))
    .run().await.unwrap();
}
