use actix_web::{App, HttpServer};
use dotenvy::dotenv;
use log::info;

use crate::games::Games;

mod env;
mod error;
mod game;
mod games;
mod routes;
mod session;

#[actix::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize logger
    env_logger::init();

    // Initialize the global games state
    Games::init();

    let port = env::from_env(env::PORT);
    info!("Starting Quizler on port {}", port);

    HttpServer::new(move || App::new().configure(routes::configure))
        .bind(("0.0.0.0", port))?
        .run()
        .await
}
