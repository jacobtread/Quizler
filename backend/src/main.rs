use crate::games::Games;
use dotenvy::dotenv;
use log::{error, info, LevelFilter};
use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    process::exit,
};

mod game;
mod games;
mod http;
mod msg;
mod session;
mod types;

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

    // Add CORS layer to the router in debug mode
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
