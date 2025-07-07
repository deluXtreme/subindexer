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
