WITH
    active_subscriptions as (
        SELECT
            active.address as contract_address,
            active.id_0 as id,
            active.subscriber_1 as subscriber,
            recipient_2 as recipient,
            amount_3 as amount,
            category_5 as category,
            frequency_4 as frequency,
            CAST(strftime('%s', 'now') AS INTEGER) as right_meow,
            creationTimestamp_6 as creation_timestamp
        FROM subscription_created active
                 LEFT JOIN unsubscribed canceled
                           ON active.id_0 = canceled.id_0
        WHERE canceled.id_0 IS NULL
    ),
    latest_redemptions AS (
        SELECT
            id,
            next_redeem_at
        FROM (
            SELECT
                id_0 as id,
                nextRedeemAt_3 as next_redeem_at,
                ROW_NUMBER() OVER (PARTITION BY id_0 ORDER BY nextRedeemAt_3 DESC) as rn
            FROM redeemed
        )
        WHERE rn = 1
    ),
    latest_recipients AS (
        SELECT
            id,
            new_recipient
        FROM (
            SELECT
                id_0 as id,
                newRecipient_2 as new_recipient,
                ROW_NUMBER() OVER (PARTITION BY id_0 ORDER BY block_number DESC) as rn
            FROM recipient_updated
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
