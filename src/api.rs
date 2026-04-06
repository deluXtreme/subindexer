use crate::{config::STALE_BLOCK_THRESHOLD, db, models};
use alloy::primitives::Address;
use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;
use sqlx::SqlitePool;

pub async fn get_redeemable(
    State(pool): State<SqlitePool>,
) -> Json<Vec<models::RedeemableSubscription>> {
    let current_timestamp = chrono::Utc::now().timestamp() as i32;
    match db::get_redeemable_subscriptions(&pool, current_timestamp).await {
        Ok(subscriptions) => Json(subscriptions),
        Err(e) => {
            // Log database errors but don't panic (tables might not exist yet)
            tracing::error!("Database error: {}", e);
            Json(Vec::new())
        }
    }
}

#[derive(Deserialize)]
pub struct SubscriptionsQuery {
    pub subscriber: Option<Address>,
    pub recipient: Option<Address>,
}

pub async fn get_subscriptions(
    State(pool): State<SqlitePool>,
    Query(query): Query<SubscriptionsQuery>,
) -> Result<Json<Vec<models::Subscription>>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match db::get_user_subscriptions(&pool, query.subscriber, query.recipient).await {
        Ok(subscriptions) => Ok(Json(subscriptions)),
        Err(e) => {
            // Log other database errors but don't panic
            tracing::error!("Database error: {}", e);
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Database error" })),
            ))
        }
    }
}

pub async fn health_check(State(pool): State<SqlitePool>) -> Json<serde_json::Value> {
    // Test database connectivity
    let db_healthy = sqlx::query("SELECT 1").execute(&pool).await.is_ok();
    let blocks_behind = db::check_liveness(&pool).await.unwrap_or(u64::MAX);
    let stale = blocks_behind > STALE_BLOCK_THRESHOLD;
    Json(serde_json::json!({
        "status": if db_healthy && !stale { "healthy" } else { "unhealthy" },
        "database": if db_healthy { "connected" } else { "disconnected" },
        "live": !stale,
        "blocks_behind": blocks_behind,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
