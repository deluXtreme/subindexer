SELECT
    active.contract_address,
    active.id,
    active.subscriber,
    recipient,
    amount,
    category,
    CAST(frequency AS INTEGER) as frequency,
    CAST(creation_timestamp AS INTEGER) as creation_timestamp
FROM subindexer_subscription_module_subscription_created active
         LEFT JOIN subindexer_subscription_module_unsubscribed canceled
                   ON active.id = canceled.id
WHERE canceled.id IS NULL
AND ($1 IS NULL OR active.subscriber = $1)
AND ($2 IS NULL OR active.recipient = $2);

