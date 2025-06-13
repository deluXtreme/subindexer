use crate::models::RedeemableSubscription;
use sqlx::PgPool;

pub async fn get_redeemable_subscriptions(
    pool: &PgPool,
    current_timestamp: i32,
) -> Result<Vec<RedeemableSubscription>, sqlx::Error> {
    let query = r#"
    WITH
    active_subscriptions as (
        SELECT active.id, subscriber, recipient, amount, "requireTrusted" as trusted
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
    )
    SELECT
      a.id,
      a.subscriber,
      a.recipient,
      a.amount,
      trusted,
      COALESCE(cast(r.next_redeem_at as Integer), 0) AS next_redeem_at
    FROM active_subscriptions a
    LEFT JOIN latest_redemptions r
      ON a.id = r.id
    WHERE COALESCE(cast(r.next_redeem_at as Integer), 0) < $1;
    "#;

    sqlx::query_as::<_, RedeemableSubscription>(query)
        .bind(current_timestamp)
        .fetch_all(pool)
        .await
}
