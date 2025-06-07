mod db;
mod models;

use axum::{extract::State, routing::get, Json, Router};
use dotenv::dotenv;
use sqlx::PgPool;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Database connection
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    // Build our application with a route
    let app = Router::new()
        .route("/redeemable", get(get_redeemable))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(pool);

    // Get port from environment variable or default to 3000
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("PORT must be a number");

    // Run it
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_redeemable(State(pool): State<PgPool>) -> Json<Vec<models::RedeemableSubscription>> {
    let current_timestamp = chrono::Utc::now().timestamp() as i32;
    let subscriptions = db::get_redeemable_subscriptions(&pool, current_timestamp)
        .await
        .expect("Failed to fetch redeemable subscriptions");
    Json(subscriptions)
}
