use actix_web::{http::StatusCode, middleware::Logger, test::{self, TestRequest}, web::{resource, Data}, App};
use env_logger::Env;

use crate::game::create::CreatedGame;

use super::{create::{create_game, CreateGameRequest}, get::get_game, manager::GamesManager};


#[actix_web::test]
async fn test_create() {
    env_logger::init_from_env(
        Env::new().default_filter_or("DEBUG")
    );

    let mut app = test::init_service(
        App::new()
            .wrap(Logger::default())
            .app_data(Data::new(GamesManager::new()))
            .service(resource("/create").post(create_game))
            .service(resource("/get").get(get_game))
    ).await;

    let items = vec![String::from("bleh"); 25];

    let create_req = TestRequest::post()
        .uri("/create")
        .set_json(CreateGameRequest{ items, size: 5 });

    let create_resp: CreatedGame = test::call_and_read_body_json(&mut app, create_req.to_request()).await;

    let get_req = TestRequest::get()
        .uri(&format!("/get?id={}", create_resp.id))
        .param("id", create_resp.id.to_string());

    let get_resp = test::call_service(&mut app, get_req.to_request()).await;

    assert!(get_resp.status() == StatusCode::OK, "status code: {}, body: {:?}", get_resp.status(), get_resp.map_into_boxed_body())
}
