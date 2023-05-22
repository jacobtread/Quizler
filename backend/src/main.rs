use crate::games::Games;
use actix_cors::Cors;
use actix_web::{App, HttpServer};
use dotenvy::dotenv;
use log::{info, LevelFilter};

mod game;
mod games;
mod http;
mod msg;
mod session;
mod types;

// Cargo package version
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[actix::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize logger
    env_logger::builder()
        .filter_module("quizler", LevelFilter::Info)
        .parse_default_env()
        .init();

    // Create the games store
    Games::init();

    let port: u16 = match std::env::var("QUIZLER_PORT") {
        Ok(value) => value
            .parse()
            .expect("Provided QUIZLER_PORT was not a valid port"),
        Err(_) => 80,
    };

    info!("Starting Quizler on port {} (v{})", port, VERSION);

    HttpServer::new(move || {
        // TODO: CORS is only required in development
        let cors = Cors::permissive();
        App::new().wrap(cors).configure(http::configure)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
