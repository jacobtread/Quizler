use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    process::exit,
};

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

    #[allow(unused_mut)]
    let mut router = http::router();

    #[cfg(debug_assertions)]
    {
        router = router.layer(tower_http::cors::CorsLayer::very_permissive());
    }

    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port));

    if let Err(err) = axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
    {
        error!("Server error: {}", err);
        exit(1);
    }
}
