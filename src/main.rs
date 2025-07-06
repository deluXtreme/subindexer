mod db;
mod models;

use axum::{Json, Router, extract::State, routing::get};
use dotenv::dotenv;
use sqlx::{PgPool, postgres::PgPoolOptions};
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
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    // Build our application with a route
    let app = Router::new()
        .route("/health", get(health_check))
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
    match db::get_redeemable_subscriptions(&pool, current_timestamp).await {
        Ok(subscriptions) => Json(subscriptions),
        Err(sqlx::Error::Database(db_err))
            if db_err.code() == Some(std::borrow::Cow::Borrowed("42P01")) =>
        {
            // Table doesn't exist yet, return empty list
            tracing::warn!("Database tables don't exist yet, returning empty list");
            Json(Vec::new())
        }
        Err(e) => {
            // Log other database errors but don't panic
            tracing::error!("Database error: {}", e);
            Json(Vec::new())
        }
    }
}

async fn health_check(State(pool): State<PgPool>) -> Json<serde_json::Value> {
    // Test database connectivity
    let db_healthy = match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => true,
        Err(_) => false,
    };

    Json(serde_json::json!({
        "status": if db_healthy { "healthy" } else { "unhealthy" },
        "database": if db_healthy { "connected" } else { "disconnected" },
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
