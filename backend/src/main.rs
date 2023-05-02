use actix::Actor;
use actix_web::{web::Data, App, HttpServer};
use dotenvy::dotenv;
use log::info;

use crate::games::Games;

mod env;
mod error;
mod game;
mod games;
mod http;
mod session;

#[actix::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize logger
    env_logger::init();

    // Create the games store
    let games = Games::start_default();
    let games = Data::new(games);

    let port = env::from_env(env::PORT);

    info!("Starting Quizler on port {}", port);

    HttpServer::new(move || {
        let games = games.clone();
        App::new().app_data(games).configure(http::configure)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
