use crate::models::RedeemableSubscription;
use sqlx::PgPool;

const REDEEMABLE_QUERY: &str = include_str!("redeemable-subscriptions.sql");

pub async fn get_redeemable_subscriptions(
    pool: &PgPool,
    current_timestamp: i32,
) -> Result<Vec<RedeemableSubscription>, sqlx::Error> {
    sqlx::query_as::<_, RedeemableSubscription>(REDEEMABLE_QUERY)
        .bind(current_timestamp)
        .fetch_all(pool)
        .await
}
