mod api;
mod db;
mod models;
mod redeem;

use axum::{Router, routing::get};
use dotenv::dotenv;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::net::SocketAddr;
use tokio::time::{Duration, interval};
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Database connection
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

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
        .with_state(pool.clone());

    // Get port from environment variable or default to 3000
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("PORT must be a number");

    // Run it
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tokio::spawn(spawn_redeemer(pool.clone()));
    axum::serve(listener, app).await.unwrap();
}

async fn spawn_redeemer(pool: PgPool) {
    let mut ticker = interval(Duration::from_secs(60 * 5)); // every 5 minutes
    tracing::info!("Cron redeemer");
    loop {
        ticker.tick().await;
        if let Err(err) = redeem::run_redeem_job(&pool).await {
            tracing::error!("Redeem job failed: {err:?}");
        }
    }
}
