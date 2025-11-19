use alloy::{
    hex,
    primitives::{Address, TxHash, U256, aliases::U192},
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
    sol,
};
use anyhow::Result;
use sqlx::SqlitePool;

use circles_pathfinder::{FindPathParams, encode_redeem_trusted_data, prepare_flow_for_contract};
use std::str::FromStr;

use crate::{
    config::STALE_BLOCK_THRESHOLD,
    db,
    eip7702::eoa_multisend,
    models::{Category, RedeemableSubscription},
    redeem::{self, SubscriptionModule::SubscriptionModuleInstance},
};

const EXPLORER_URL: &str = "https://gnosisscan.io/tx";

pub async fn run_redeem_job(
    rpc_url: &str,
    pool: &SqlitePool,
    signer: &PrivateKeySigner,
) -> Result<()> {
    tracing::info!("Running redeem job with signer: {:?}", signer.address());
    // Ensure indexer liveness.
    let blocks_behind = db::check_liveness(pool).await?;
    if blocks_behind > STALE_BLOCK_THRESHOLD {
        tracing::warn!(
            "Stale indexer: {blocks_behind} blocks behind latest. transaction may fail...",
        );
    }

    let current_timestamp = chrono::Utc::now().timestamp() as i32;
    let subscriptions = db::get_redeemable_subscriptions(pool, current_timestamp).await?;
    redeem::redeem_payments(rpc_url, signer.clone(), subscriptions).await
}

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    SubscriptionModule,
    "abis/SubscriptionModule.abi.json"
);

const CIRCLES_RPC: &str = "https://rpc.aboutcircles.com/";

pub async fn redeem_singular(
    rpc_url: &str,
    signer: PrivateKeySigner,
    subscription: &RedeemableSubscription,
) -> Result<TxHash> {
    let subscription_module = subscription.contract_address.parse::<Address>().unwrap();

    let provider = ProviderBuilder::new()
        .wallet(signer)
        .connect_http(rpc_url.parse()?);
    let contract = SubscriptionModule::new(subscription_module, &provider);
    let tx = encode_tx(contract, subscription).await?;
    let tx_hash = provider.send_transaction(tx).await?.watch().await?;
    Ok(tx_hash)
}

async fn redeem_multi(
    rpc_url: &str,
    signer: PrivateKeySigner,
    subscriptions: Vec<RedeemableSubscription>,
) -> Result<TxHash> {
    let provider = ProviderBuilder::new().connect_http(rpc_url.parse()?);
    // TODO: this could be constructed asynchronously. Might be rate limited by the RPC.
    // futures::future::try_join_all
    // subscriptions.iter().map(|rs| async {
    //     let sub_mod = rs.contract_address.parse()?;
    //     let contract = SubscriptionModule::new(sub_mod, &provider);
    //     encode_tx(contract, rs).await
    // })
    let mut tx_requests = vec![];
    for rs in subscriptions.iter() {
        let sub_mod = rs.contract_address.parse()?;
        let contract = SubscriptionModule::new(sub_mod, &provider);
        let tx_data = encode_tx(contract, rs).await?;
        tx_requests.push(tx_data);
    }
    eoa_multisend(rpc_url, signer, tx_requests).await
}

pub async fn redeem_payments(
    rpc_url: &str,
    signer: PrivateKeySigner,
    subscriptions: Vec<RedeemableSubscription>,
) -> Result<()> {
    if subscriptions.is_empty() {
        tracing::info!("No subscriptions to redeem");
        return Ok(());
    }
    tracing::info!(
        "Redeeming {} subscription(s): {}",
        subscriptions.len(),
        subscriptions
            .iter()
            .map(|s| format!("0x{}", hex::encode(&s.id)))
            .collect::<Vec<String>>()
            .join(", ")
    );
    let hash = if subscriptions.len() == 1 {
        redeem_singular(rpc_url, signer, &subscriptions[0]).await?
    } else {
        redeem_multi(rpc_url, signer, subscriptions).await?
    };
    tracing::info!("Redeemed at: {EXPLORER_URL}/{hash}");
    Ok(())
}

async fn encode_tx<P: Provider>(
    contract: SubscriptionModuleInstance<P>,
    subscription: &RedeemableSubscription,
) -> Result<TransactionRequest> {
    let id = U256::from_str_radix(subscription.id.trim_start_matches("0x"), 16)?;
    let tx;
    if subscription.category != Category::Trusted {
        tx = contract.redeem(id.into(), vec![].into());
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
        tracing::debug!("Encoded CallData: 0x{}", hex::encode(&data));
        tx = contract.redeem(id.into(), data.into());
    }
    Ok(tx.into_transaction_request())
}
