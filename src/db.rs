use crate::models::RedeemableSubscription;
use sqlx::PgPool;

pub async fn get_redeemable_subscriptions(
    pool: &PgPool,
    current_timestamp: i32,
) -> Result<Vec<RedeemableSubscription>, sqlx::Error> {
    // TODO: Use the SQL file instead of the inline query (requires fs).
    // let query = fs::read_to_string("src/redeemable-subscriptions.sql")
    // .expect("Failed to read redeemable-subscriptions.sql");
    let query = r#"
    WITH
        active_subscriptions as (
            SELECT
                active.id,
                active.subscriber,
                recipient,
                amount,
                category
            FROM subindexer_subscription_module.subscription_created active
                    LEFT JOIN subindexer_subscription_module.unsubscribed canceled
                              ON active.id = canceled.id
            WHERE canceled.id IS NULL
        ),
        latest_redemptions AS (
            SELECT DISTINCT ON (id)
                id,
                next_redeem_at
            FROM subindexer_subscription_module.redeemed
            ORDER BY id, next_redeem_at DESC
        ),
        latest_recipients AS (
            SELECT DISTINCT ON (id)
                id,
                new_recipient
            FROM subindexer_subscription_module.recipient_updated
            ORDER BY id, block_number DESC
        )
    SELECT
        a.id,
        a.subscriber,
        COALESCE(rp.new_recipient, a.recipient) AS recipient,
        a.amount,
        category,
        COALESCE(cast(r.next_redeem_at as Integer), 0) AS next_redeem_at
    FROM active_subscriptions a
            LEFT JOIN latest_redemptions r
                      ON a.id = r.id
            LEFT JOIN latest_recipients rp
                      ON a.id = rp.id
    WHERE COALESCE(cast(r.next_redeem_at as Integer), 0) < $1;
    "#;

    sqlx::query_as::<_, RedeemableSubscription>(query)
        .bind(current_timestamp)
        .fetch_all(pool)
        .await
}
