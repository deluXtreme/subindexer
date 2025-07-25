SELECT
    active.id,
    active.subscriber,
    recipient,
    amount,
    category,
    frequency::INTEGER as frequency,
    creation_timestamp::INTEGER as creation_timestamp
FROM subindexer_subscription_module.subscription_created active
          LEFT JOIN subindexer_subscription_module.unsubscribed canceled
                    ON active.id = canceled.id
WHERE canceled.id IS NULL
AND active.subscriber = $1;
