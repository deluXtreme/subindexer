WITH
    active_subscriptions as (
        SELECT
            active.contract_address,
            active.id,
            active.subscriber,
            recipient,
            amount,
            category,
            frequency,
            CAST(strftime('%s', 'now') AS INTEGER) as right_meow,
            creation_timestamp
        FROM subindexer_subscription_module_subscription_created active
                 LEFT JOIN subindexer_subscription_module_unsubscribed canceled
                           ON active.id = canceled.id
        WHERE canceled.id IS NULL
    ),
    latest_redemptions AS (
        SELECT
            id,
            next_redeem_at
        FROM (
            SELECT
                id,
                next_redeem_at,
                ROW_NUMBER() OVER (PARTITION BY id ORDER BY next_redeem_at DESC) as rn
            FROM subindexer_subscription_module_redeemed
        )
        WHERE rn = 1
    ),
    latest_recipients AS (
        SELECT
            id,
            new_recipient
        FROM (
            SELECT
                id,
                new_recipient,
                ROW_NUMBER() OVER (PARTITION BY id ORDER BY block_number DESC) as rn
            FROM subindexer_subscription_module_recipient_updated
        )
        WHERE rn = 1
    ),
    upcoming AS (
        SELECT
            a.contract_address,
            a.id,
            a.subscriber,
            COALESCE(rp.new_recipient, a.recipient) AS recipient,
            amount,
            -- Redeemable Periods: cf https://github.com/deluXtreme/subi-contracts/blob/65455f02e3e7a49654c51b9b5e805cccc1032168/src/SubscriptionModule.sol#L154-L158
            CAST((right_meow - COALESCE(CAST(r.next_redeem_at AS INTEGER), CAST(creation_timestamp AS INTEGER)) + CAST(frequency AS INTEGER)) / CAST(a.frequency AS INTEGER) AS INTEGER) as periods,
            category,
            CAST(COALESCE(r.next_redeem_at, creation_timestamp) AS INTEGER) AS next_redeem_at
        FROM active_subscriptions a
                LEFT JOIN latest_redemptions r
                        ON a.id = r.id
                LEFT JOIN latest_recipients rp
                        ON a.id = rp.id
    )
SELECT * from upcoming
WHERE next_redeem_at < $1;
