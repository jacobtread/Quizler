use std::{net::Ipv4Addr, process::exit};

use dotenvy::dotenv;
use log::{error, info, LevelFilter};

use crate::games::Games;

pub(crate) mod game;
pub(crate) mod games;
pub(crate) mod http;
pub(crate) mod msg;
pub(crate) mod session;
pub(crate) mod types;

// Cargo package version
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
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

    let server = HttpServer::new(move || {
        // Include CORS support for debug builds
        #[cfg(debug_assertions)]
        {
            use actix_cors::Cors;
            let cors = Cors::permissive();
            App::new().wrap(cors).configure(http::configure)
        }
        // Release builds don't require CORS
        #[cfg(not(debug_assertions))]
        {
            App::new().configure(http::configure)
        }
    });

    let server = match server.bind((Ipv4Addr::UNSPECIFIED, port)) {
        Ok(value) => value,
        Err(error) => {
            error!("Failed to start server: {}", error);
            exit(1);
        }
    };

    if let Err(error) = server.run().await {
        error!("Server error: {}", error);
        exit(1);
    }
}
