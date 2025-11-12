use alloy::{
    hex,
    primitives::{Address, U256, aliases::U192},
    providers::{Provider, ProviderBuilder},
    signers::local::PrivateKeySigner,
    sol,
};
use sqlx::PgPool;

use circles_pathfinder::{FindPathParams, encode_redeem_trusted_data, prepare_flow_for_contract};
use std::{error::Error, str::FromStr};

use crate::{
    db,
    models::{Category, RedeemableSubscription},
    redeem,
};

const STALE_BLOCK_THRESHOLD: u64 = 100;

pub async fn run_redeem_job(
    rpc_url: &str,
    pool: &PgPool,
    signer: &PrivateKeySigner,
) -> Result<(), Box<dyn Error>> {
    tracing::info!("Running redeem job with signer: {:?}", signer.address());
    // Ensure indexer liveness.
    let last_synced_block = db::get_last_synced_block(pool).await?;
    let provider = ProviderBuilder::new().connect_http(rpc_url.parse()?);
    let latest_block = provider.get_block_number().await?;
    if latest_block <= last_synced_block + STALE_BLOCK_THRESHOLD {
        tracing::error!(
            "Stale indexer: latest block {} <= last synced block {}",
            latest_block,
            last_synced_block
        );
        return Ok(());
    }

    let current_timestamp = chrono::Utc::now().timestamp() as i32;
    let subscriptions = db::get_redeemable_subscriptions(pool, current_timestamp).await?;
    tracing::info!("Found {} subscriptions", subscriptions.len());
    for subscription in subscriptions {
        redeem::redeem_payment(rpc_url, signer.clone(), subscription).await?;
    }
    Ok(())
}

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    SubscriptionModule,
    "abis/SubscriptionModule.abi.json"
);

const CIRCLES_RPC: &str = "https://rpc.aboutcircles.com/";

pub async fn redeem_payment(
    rpc_url: &str,
    signer: PrivateKeySigner,
    subscription: RedeemableSubscription,
) -> Result<bool, Box<dyn std::error::Error>> {
    let subscription_module = "0xcEbE4B6d50Ce877A9689ce4516Fe96911e099A78"
        .parse::<Address>()
        .unwrap();

    let provider = ProviderBuilder::new()
        .wallet(signer)
        .connect_http(rpc_url.parse()?);
    let contract = SubscriptionModule::new(subscription_module, provider);
    let id = U256::from_be_slice(&subscription.id);
    let tx;
    tracing::info!(
        "Redeeming: {}",
        serde_json::to_string_pretty(&subscription).unwrap()
    );
    if subscription.category != Category::Trusted {
        tx = contract.redeem(id.into(), vec![].into()).send().await?;
    } else {
        let amount = U192::from_str(&subscription.amount)?;
        let periods = U192::from(subscription.periods as u64);
        let params = FindPathParams {
            from: subscription.subscriber.parse::<Address>()?,
            to: subscription.recipient.parse::<Address>()?,
            target_flow: amount * periods,
            use_wrapped_balances: Some(true),
            from_tokens: None,
            to_tokens: None,
            exclude_from_tokens: None,
            exclude_to_tokens: None,
        };

        // This automatically:
        // - Finds the optimal path
        // - Creates the flow matrix
        // - Converts to contract-compatible types
        // - Handles flow balancing
        let path_data = prepare_flow_for_contract(CIRCLES_RPC, params).await?;
        let data = encode_redeem_trusted_data(
            path_data.flow_vertices,
            path_data.flow_edges,
            path_data.streams,
            path_data.packed_coordinates,
            path_data.source_coordinate,
        );
        tracing::info!("Encoded CallData: 0x{}", hex::encode(&data));
        tx = contract.redeem(id.into(), data.into()).send().await?;
    }
    tracing::info!(
        "Redeemed 0x{} at: https://gnosisscan.io/tx/{}",
        hex::encode(subscription.id),
        tx.tx_hash()
    );
    let receipt = tx.get_receipt().await?;
    tracing::info!("Receipt Status: {:?}", receipt.status());
    Ok(true)
}
