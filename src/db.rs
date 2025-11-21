use crate::models::{RedeemableSubscription, Subscription};
use alloy::{
    primitives::Address,
    providers::{Provider, ProviderBuilder},
};
use anyhow::{Context, Result};
use sqlx::SqlitePool;

const REDEEMABLE_QUERY: &str = include_str!("queries/redeemable.sql");

pub async fn get_redeemable_subscriptions(
    pool: &SqlitePool,
    current_timestamp: i32,
) -> Result<Vec<RedeemableSubscription>, sqlx::Error> {
    sqlx::query_as::<_, RedeemableSubscription>(REDEEMABLE_QUERY)
        .bind(current_timestamp)
        .fetch_all(pool)
        .await
}

async fn get_last_synced_block(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "SELECT block FROM rindexer_internal_latest_block WHERE network = 'gnosis'",
    )
    .fetch_one(pool)
    .await
    .map(|result| result as u64)
}

// Returns number of blocks behind latest
pub async fn check_liveness(pool: &SqlitePool) -> Result<u64> {
    let last_synced_block = get_last_synced_block(pool)
        .await
        .context("Failed to get last synced block")?;

    // Use a different RPC as indexer (because node may not be synced.)
    let provider = ProviderBuilder::new().connect_http("https://rpc.gnosischain.com/".parse()?);
    let latest_block = provider.get_block_number().await?;
    Ok(latest_block - last_synced_block)
}

const USER_SUBSCRIPTIONS_QUERY: &str = include_str!("queries/user_subscriptions.sql");

pub async fn get_user_subscriptions(
    pool: &SqlitePool,
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
