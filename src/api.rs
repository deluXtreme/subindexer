use crate::{db, models};
use alloy::primitives::Address;
use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;
use sqlx::PgPool;

pub async fn get_redeemable(
    State(pool): State<PgPool>,
) -> Json<Vec<models::RedeemableSubscription>> {
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

#[derive(Deserialize)]
pub struct SubscriptionsQuery {
    pub subscriber: Option<Address>,
    pub recipient: Option<Address>,
}

pub async fn get_subscriptions(
    State(pool): State<PgPool>,
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

pub async fn health_check(State(pool): State<PgPool>) -> Json<serde_json::Value> {
    // Test database connectivity
    let db_healthy = (sqlx::query("SELECT 1").execute(&pool).await).is_ok();
    let latest_synced_block = db::get_last_synced_block(&pool).await.is_ok();
    Json(serde_json::json!({
        "status": if db_healthy && latest_synced_block { "healthy" } else { "unhealthy" },
        "database": if db_healthy { "connected" } else { "disconnected" },
        "latest_synced_block": latest_synced_block,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
