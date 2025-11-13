use crate::models::{RedeemableSubscription, Subscription};
use alloy::primitives::Address;
use sqlx::PgPool;

const REDEEMABLE_QUERY: &str = include_str!("queries/redeemable.sql");

pub async fn get_redeemable_subscriptions(
    pool: &PgPool,
    current_timestamp: i32,
) -> Result<Vec<RedeemableSubscription>, sqlx::Error> {
    sqlx::query_as::<_, RedeemableSubscription>(REDEEMABLE_QUERY)
        .bind(current_timestamp)
        .fetch_all(pool)
        .await
}

pub async fn get_last_synced_block(pool: &PgPool) -> Result<u64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "SELECT block::bigint FROM rindexer_internal.latest_block WHERE network = 'gnosis'",
    )
    .fetch_one(pool)
    .await
    .map(|result| result as u64)
}

const USER_SUBSCRIPTIONS_QUERY: &str = include_str!("queries/user_subscriptions.sql");

pub async fn get_user_subscriptions(
    pool: &PgPool,
    subscriber: Option<Address>,
    recipient: Option<Address>,
) -> Result<Vec<Subscription>, sqlx::Error> {
    tracing::info!(
        "Getting user subscriptions for subscriber: {:?}, recipient: {:?}",
        subscriber,
        recipient
    );

    let subscriber_hex = subscriber.map(|s| format!("0x{}", alloy::hex::encode(s)));
    let recipient_hex = recipient.map(|r| format!("0x{}", alloy::hex::encode(r)));

    sqlx::query_as::<_, Subscription>(USER_SUBSCRIPTIONS_QUERY)
        .bind(subscriber_hex)
        .bind(recipient_hex)
        .fetch_all(pool)
        .await
}
