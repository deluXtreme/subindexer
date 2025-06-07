use crate::models::RedeemableSubscription;
use sqlx::PgPool;

pub async fn get_redeemable_subscriptions(
    pool: &PgPool,
    current_timestamp: i32,
) -> Result<Vec<RedeemableSubscription>, sqlx::Error> {
    let query = r#"
    WITH
    active_subscriptions as (
        SELECT active.sub_id, active.module, subscriber, recipient, amount
        FROM subindexer_subscription_manager.subscription_created active
        LEFT JOIN subindexer_subscription_manager.subscription_cancelled canceled
        ON active.sub_id = canceled.sub_id AND active.module = canceled.module
        WHERE canceled.sub_id IS NULL
    ),
    latest_redemptions AS (
      SELECT DISTINCT ON (sub_id, module)
        sub_id,
        module,
        next_redeem_at
      FROM subindexer_subscription_manager.redeemed
      ORDER BY sub_id, module, next_redeem_at DESC
    )
    SELECT
      a.sub_id,
      a.module,
      a.subscriber,
      a.recipient,
      a.amount,
      COALESCE(cast(r.next_redeem_at as Integer), 0) AS next_redeem_at
    FROM active_subscriptions a
    LEFT JOIN latest_redemptions r
      ON a.sub_id = r.sub_id AND a.module = r.module
    WHERE COALESCE(cast(r.next_redeem_at as Integer), 0) < $1;
    "#;

    sqlx::query_as::<_, RedeemableSubscription>(query)
        .bind(current_timestamp)
        .fetch_all(pool)
        .await
}
