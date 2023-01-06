use actix_web::{App, HttpServer};
use dotenvy::dotenv;
use log::info;

mod env;
mod game;
mod games;
mod session;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize logger
    env_logger::init();

    let port = env::from_env(env::PORT);
    info!("Starting Quizler on port {}", port);
    HttpServer::new(|| App::new())
        .bind(("0.0.0.0", port))?
        .run()
        .await
}
