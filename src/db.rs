use crate::models::{RedeemableSubscription, Subscription};
use alloy::primitives::Address;
use sqlx::Error as SqlxError;
use sqlx::PgPool;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Sqlx(#[from] SqlxError),

    #[error("Bad Request: {0}")]
    BadRequest(String),
}

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

const RECIPIENT_QUERY: &str = include_str!("queries/recipient.sql");
const SUBSCRIBER_QUERY: &str = include_str!("queries/subscriber.sql");
const RECIPIENT_SUBSCRIBER_QUERY: &str = include_str!("queries/recipient_subscriber.sql");

pub async fn get_user_subscriptions(
    pool: &PgPool,
    subscriber: Option<Address>,
    recipient: Option<Address>,
) -> Result<Vec<Subscription>, AppError> {
    tracing::info!(
        "Getting user subscriptions for subscriber: {:?}, recipient: {:?}",
        subscriber,
        recipient
    );
    match (subscriber, recipient) {
        (None, None) => {
            // Early exit to avoid unnecessary database queries.
            Err(AppError::BadRequest(
                "At least one of subscriber or recipient must be specified.".to_string(),
            ))
        }
        (Some(subscriber), Some(recipient)) => {
            let subscriber_hex = format!("0x{}", alloy::hex::encode(subscriber));
            let recipient_hex = format!("0x{}", alloy::hex::encode(recipient));
            sqlx::query_as::<_, Subscription>(RECIPIENT_SUBSCRIBER_QUERY)
                .bind(subscriber_hex)
                .bind(recipient_hex)
                .fetch_all(pool)
                .await
                .map_err(AppError::Sqlx)
        }
        (Some(subscriber), None) => {
            let subscriber_hex = format!("0x{}", alloy::hex::encode(subscriber));
            sqlx::query_as::<_, Subscription>(SUBSCRIBER_QUERY)
                .bind(subscriber_hex)
                .fetch_all(pool)
                .await
                .map_err(AppError::Sqlx)
        }
        (None, Some(recipient)) => {
            let recipient_hex = format!("0x{}", alloy::hex::encode(recipient));
            sqlx::query_as::<_, Subscription>(RECIPIENT_QUERY)
                .bind(recipient_hex)
                .fetch_all(pool)
                .await
                .map_err(AppError::Sqlx)
        }
    }
}
