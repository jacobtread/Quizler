use crate::games::Games;
use actix::Actor;
use actix_web::{web::Data, App, HttpServer};
use dotenvy::dotenv;
use log::info;

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

    let port: u16 = std::env::var("QUIZLER_PORT")
        .map(|value| {
            value
                .parse()
                .expect("Provided QUIZLER_PORT was not a valid port")
        })
        .unwrap_or(80);

    info!("Starting Quizler on port {}", port);

    HttpServer::new(move || {
        let games = games.clone();
        App::new().app_data(games).configure(http::configure)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
