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

    // Spawn the cleanup future
    tokio::spawn(Games::tick_cleanup());

    let port: u16 = std::env::var("QUIZLER_PORT")
        .map(|value| {
            value
                .parse::<u16>()
                .expect("Provided QUIZLER_PORT was not a valid port")
        })
        .unwrap_or(80);

    info!("Starting Quizler on port {} (v{})", port, VERSION);

    #[allow(unused_mut)]
    let mut router = http::router();

    #[cfg(debug_assertions)]
    {
        // Add CORS and tracing layer to the router in debug mode
        router = router
            .layer(tower_http::cors::CorsLayer::very_permissive())
            .layer(tower_http::trace::TraceLayer::new_for_http());
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
