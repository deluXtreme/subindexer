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
                amount::NUMERIC as amount,
                category,
                frequency::NUMERIC as frequency,
                creation_timestamp::NUMERIC as creation_timestamp
            FROM subindexer_subscription_module.subscription_created active
                    LEFT JOIN subindexer_subscription_module.unsubscribed canceled
                            ON active.id = canceled.id
            WHERE canceled.id IS NULL
        ),
        latest_redemptions AS (
            SELECT DISTINCT ON (id)
                id,
                next_redeem_at,
                block_number as last_redeemed
            FROM subindexer_subscription_module.redeemed
            ORDER BY id, next_redeem_at DESC
        ),
        latest_recipients AS (
            SELECT DISTINCT ON (id)
                id,
                new_recipient
            FROM subindexer_subscription_module.recipient_updated
            ORDER BY id, block_number DESC
        ),
        upcomming AS (
            SELECT
                a.id,
                a.subscriber,
                COALESCE(rp.new_recipient, a.recipient) AS recipient,
                -- Redeemable Amount: cf https://github.com/deluXtreme/subi-contracts/blob/65455f02e3e7a49654c51b9b5e805cccc1032168/src/SubscriptionModule.sol#L154-L158
                (FLOOR((FLOOR(EXTRACT(EPOCH FROM now()))::NUMERIC - COALESCE(cast(r.last_redeemed as Integer), creation_timestamp)) / a.frequency) * a.amount)::TEXT as amount,
                category,
                COALESCE(cast(r.next_redeem_at as Integer), creation_timestamp) AS next_redeem_at
            FROM active_subscriptions a
                    LEFT JOIN latest_redemptions r
                            ON a.id = r.id
                    LEFT JOIN latest_recipients rp
                            ON a.id = rp.id
        )
    SELECT * from upcomming
    WHERE next_redeem_at < $1;
    "#;

    sqlx::query_as::<_, RedeemableSubscription>(query)
        .bind(current_timestamp)
        .fetch_all(pool)
        .await
}
