use crate::{db, models};
use axum::{Json, extract::State};
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

pub async fn health_check(State(pool): State<PgPool>) -> Json<serde_json::Value> {
    // Test database connectivity
    let db_healthy = (sqlx::query("SELECT 1").execute(&pool).await).is_ok();

    Json(serde_json::json!({
        "status": if db_healthy { "healthy" } else { "unhealthy" },
        "database": if db_healthy { "connected" } else { "disconnected" },
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
