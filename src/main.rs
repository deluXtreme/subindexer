mod api;
mod config;
mod db;
mod models;
mod redeem;

use axum::{Router, routing::get};
use config::Config;
use dotenv::dotenv;
use std::net::SocketAddr;
use tokio::time::{Duration, interval};
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    let config = Config::from_env().await;
    config.log_non_secrets();

    // Build our application with a route
    let app = Router::new()
        .route("/health", get(api::health_check))
        .route("/redeemable", get(api::get_redeemable))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(config.pool.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.api_port));
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tokio::spawn(spawn_redeemer(config));
    axum::serve(listener, app).await.unwrap();
}

async fn spawn_redeemer(config: Config) {
    let mut ticker = interval(Duration::from_secs(config.redeem_interval));
    tracing::info!("Cron redeemer");
    loop {
        ticker.tick().await;
        if let Err(err) =
            redeem::run_redeem_job(&config.rpc_url, &config.pool, &config.redeemer).await
        {
            tracing::error!("Redeem job failed: {err:?}");
        }
    }
}
