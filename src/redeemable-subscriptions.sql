WITH
    active_subscriptions as (
        SELECT
            active.id,
            active.subscriber,
            recipient,
            amount::NUMERIC as amount,
            category,
            frequency::NUMERIC as frequency,
            creation_timestamp::INTEGER as creation_timestamp
        FROM subindexer_subscription_module.subscription_created active
                 LEFT JOIN subindexer_subscription_module.unsubscribed canceled
                           ON active.id = canceled.id
        WHERE canceled.id IS NULL
    ),
    latest_redemptions AS (
        SELECT DISTINCT ON (id)
            id,
            next_redeem_at::INTEGER as next_redeem_at,
            block_number::INTEGER as last_redeemed
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
    upcoming AS (
        SELECT
            a.id,
            a.subscriber,
            COALESCE(rp.new_recipient, a.recipient) AS recipient,
            -- Redeemable Amount: cf https://github.com/deluXtreme/subi-contracts/blob/65455f02e3e7a49654c51b9b5e805cccc1032168/src/SubscriptionModule.sol#L154-L158
            (FLOOR((FLOOR(EXTRACT(EPOCH FROM now()))::NUMERIC - COALESCE(r.last_redeemed, creation_timestamp - frequency)) / a.frequency) * a.amount)::TEXT as amount,
            category,
            COALESCE(r.next_redeem_at, creation_timestamp) AS next_redeem_at
        FROM active_subscriptions a
                LEFT JOIN latest_redemptions r
                        ON a.id = r.id
                LEFT JOIN latest_recipients rp
                        ON a.id = rp.id
    )
SELECT * from upcoming
WHERE next_redeem_at < $1;
