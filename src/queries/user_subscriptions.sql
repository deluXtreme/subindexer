SELECT
    active.address as contract_address,
    active.id_0 as id,
    active.subscriber_1 as subscriber,
    recipient_2 as recipient,
    amount_3 as amount,
    category_5 as category,
    CAST(frequency_4 AS INTEGER) as frequency,
    CAST(creationTimestamp_6 AS INTEGER) as creation_timestamp
FROM subscription_created active
         LEFT JOIN unsubscribed canceled
                   ON active.id_0 = canceled.id_0
WHERE canceled.id_0 IS NULL
AND ($1 IS NULL OR active.subscriber_1 = $1)
AND ($2 IS NULL OR active.recipient_2 = $2);

