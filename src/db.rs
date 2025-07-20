use crate::models::RedeemableSubscription;
use sqlx::PgPool;
use tokio::fs;

const QUERY_PATH: &str = "src/redeemable-subscriptions.sql";

pub async fn get_redeemable_subscriptions(
    pool: &PgPool,
    current_timestamp: i32,
) -> Result<Vec<RedeemableSubscription>, sqlx::Error> {
    let query = fs::read_to_string(QUERY_PATH)
        .await
        .unwrap_or_else(|e| panic!("Failed to read SQL file at '{QUERY_PATH}': {e}"));

    sqlx::query_as::<_, RedeemableSubscription>(&query)
        .bind(current_timestamp)
        .fetch_all(pool)
        .await
}
