use alloy::{
    hex,
    primitives::{TxHash, U256},
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
    redeem::SubscriptionModule::SubscriptionModuleInstance,
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
    redeem_payments(rpc_url, signer.clone(), subscriptions).await
}

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    interface SubscriptionModule {
        function redeem(bytes32 id, bytes calldata data) external;
    }
);

const CIRCLES_RPC: &str = "https://rpc.aboutcircles.com/";

pub async fn redeem_singular(
    rpc_url: &str,
    signer: PrivateKeySigner,
    subscription: &RedeemableSubscription,
) -> Result<TxHash> {
    let provider = ProviderBuilder::new()
        .wallet(signer)
        .connect_http(rpc_url.parse()?);
    let contract = SubscriptionModule::new(subscription.contract_address, &provider);
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
        let contract = SubscriptionModule::new(rs.contract_address, &provider);
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
            .map(|s| format!("0x{}", hex::encode(s.id)))
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
    let tx;
    if subscription.category != Category::Trusted {
        tx = contract.redeem(subscription.id, vec![].into());
    } else {
        let amount = U256::from_str(&subscription.amount)?;
        let periods = U256::from(subscription.periods as u64);
        let params = FindPathParams {
            from: subscription.subscriber,
            to: subscription.recipient,
            target_flow: amount * periods,
            use_wrapped_balances: Some(false),
            from_tokens: None,
            to_tokens: None,
            exclude_from_tokens: None,
            exclude_to_tokens: None,
            simulated_balances: None,
            simulated_trusts: None,
            max_transfers: None,
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
        tx = contract.redeem(subscription.id, data.into());
    }
    Ok(tx.into_transaction_request())
}

#[cfg(test)]
mod e2e {
    //! End-to-end, read-only simulation of the redeem flow.
    //!
    //! Each redeemable subscription is encoded with the *production* `encode_tx`
    //! (incl. the Circles pathfinder for trusted subs) and then `eth_call`ed
    //! individually against live Gnosis chain — no private key, no state change.
    //! This surfaces the real per-subscription revert reason instead of the
    //! opaque `-32000 execution reverted` the batch job logs.
    //!
    //! Run with:
    //!   cargo test --package subindexer redeem_each_individually -- --ignored --nocapture
    use super::*;
    use alloy::primitives::{Address, Bytes, address};

    const GNOSIS_RPC: &str = "https://rpc.gnosischain.com";
    const HUB: Address = address!("0xc12C1E50ABB450d6205Ea2C3Fa861b3B834d13e8");
    // The relayer EOA. Only its address is needed for `eth_call` (no signing).
    const CALLER: Address = address!("0x8ba11add9bb5b60028eff90a14f0ae20b429ce8f");

    sol!(
        #[sol(rpc)]
        interface SubView {
            function isValidOrRedeemable(bytes32 id) external view returns (uint256);
        }
        #[sol(rpc)]
        interface IHub {
            function balanceOf(address account, uint256 id) external view returns (uint256);
        }
    );

    /// Map a 4-byte revert selector to a human-readable contract error.
    fn decode_revert(data: &Bytes) -> String {
        if data.len() < 4 {
            return format!("revert 0x{}", hex::encode(data));
        }
        let selector = hex::encode(&data[..4]);
        let name = match selector.as_str() {
            "acfdb444" => "ExecutionFailed()", // inner Safe call returned false (insufficient CRC)
            "2c5211c6" => "InvalidAmount()",   // flow amount != on-chain redeemable
            "c8501880" => "NotRedeemable()",   // not yet due
            "9c8d2cd2" => "InvalidRecipient()",
            "891d7c4c" => "InvalidSubscriber()",
            "bb5c88e9" => "InvalidStreamSource()",
            "00e6a170" => "IdentifierNonexistent()",
            "d67592f6" => "InvalidCategory()",
            other => return format!("revert <unknown 0x{other}>"),
        };
        name.to_string()
    }

    fn crc(wei: U256) -> f64 {
        wei.to::<u128>() as f64 / 1e18
    }

    /// The four subscriptions currently stuck in the daily batch job.
    /// Identity (id/addresses/category) is stable; `periods` is refreshed
    /// from chain at runtime so the trusted target flow matches the contract.
    fn fixture() -> Vec<RedeemableSubscription> {
        let module = address!("0xcebe4b6d50ce877a9689ce4516fe96911e099a78");
        let amount = "10000000000000000".to_string(); // 0.01 CRC
        let mk = |id: &str, subscriber: &str, recipient: &str, category: Category| {
            RedeemableSubscription {
                contract_address: module,
                id: id.parse().unwrap(),
                subscriber: subscriber.parse().unwrap(),
                recipient: recipient.parse().unwrap(),
                amount: amount.clone(),
                periods: 0, // refreshed on-chain below
                category,
            }
        };
        vec![
            mk(
                "0x50ede65601819b8885dc3dbf4676204fcd318c26b8281d82af20f69d55b4ca75",
                "0xcf6dc192dc292d5f2789da2db02d6dd4f41f4214",
                "0x6b69683c8897e3d18e74b1ba117b49f80423da5d",
                Category::Trusted,
            ),
            mk(
                "0x39defdf4a99087583e6cb5d58c5f91cf443caca59e2ef0f10e21acd9bea73fd5",
                "0xede0c2e70e8e2d54609c1bdf79595506b6f623fe",
                "0x6b69683c8897e3d18e74b1ba117b49f80423da5d",
                Category::Untrusted,
            ),
            mk(
                "0x5863714d5f7d8ab92945785ee0b2701ff4978342d7681f0d2df93ace4cddb916",
                "0x6b69683c8897e3d18e74b1ba117b49f80423da5d",
                "0xede0c2e70e8e2d54609c1bdf79595506b6f623fe",
                Category::Trusted,
            ),
            mk(
                "0xb980f6e65bfa94518de8c2ce77a48c6f4507c75cdc97ce05fc541f0101bc5c8e",
                "0x6b69683c8897e3d18e74b1ba117b49f80423da5d",
                "0xcf6dc192dc292d5f2789da2db02d6dd4f41f4214",
                Category::Trusted,
            ),
        ]
    }

    #[tokio::test]
    #[ignore = "e2e: hits live Gnosis RPC + Circles pathfinder; run with --ignored --nocapture"]
    async fn redeem_each_individually() -> Result<()> {
        let provider = ProviderBuilder::new().connect_http(GNOSIS_RPC.parse()?);
        let hub = IHub::new(HUB, &provider);
        let amount_wei = U256::from(10_000_000_000_000_000u64); // 0.01 CRC

        println!("\n{:=<108}", "");
        println!(
            "{:<10} {:<12} {:>10} {:>14} {:>14}   eth_call result",
            "category", "id", "periods", "need(CRC)", "have(CRC)"
        );
        println!("{:-<108}", "");

        // (category, insufficient_balance, outcome)
        let mut verdicts: Vec<(Category, bool, String)> = vec![];

        for mut sub in fixture() {
            let module = sub.contract_address;
            let subscriber = sub.subscriber;
            let id = sub.id;

            // Exact redeemable amount = on-chain periods * amount.
            let redeemable = SubView::new(module, &provider)
                .isValidOrRedeemable(id)
                .call()
                .await?;
            sub.periods = (redeemable / amount_wei).to::<u64>() as i32;

            // Subscriber's own personal CRC (token id = uint256(uint160(avatar))).
            let token_id = U256::from_be_slice(subscriber.as_slice());
            let balance = hub.balanceOf(subscriber, token_id).call().await?;
            let insufficient = balance < redeemable;

            // Build the tx exactly as production does, then simulate (no send).
            let contract = SubscriptionModule::new(module, &provider);
            let outcome = match encode_tx(contract, &sub).await {
                Err(e) => format!("BUILD FAILED (pathfinder/encode): {e}"),
                Ok(mut tx) => {
                    tx.from = Some(CALLER);
                    match provider.call(tx).await {
                        Ok(_) => "PASS — would succeed".to_string(),
                        Err(e) => match e.as_error_resp().and_then(|r| r.as_revert_data()) {
                            Some(data) => format!("FAIL — {}", decode_revert(&data)),
                            None => format!("FAIL — {e}"),
                        },
                    }
                }
            };

            println!(
                "{:<10} {:<12} {:>10} {:>14.6} {:>14.6}   {}",
                format!("{:?}", sub.category),
                &format!("{id:#x}")[..10],
                sub.periods,
                crc(redeemable),
                crc(balance),
                outcome,
            );
            verdicts.push((sub.category, insufficient, outcome));
        }
        println!("{:=<108}", "");

        // Hypothesis proof: an *untrusted* redeem moves the subscriber's own
        // personal CRC directly. With a dust balance it must revert ExecutionFailed.
        for (category, insufficient, outcome) in &verdicts {
            if *category == Category::Untrusted && *insufficient {
                assert!(
                    outcome.contains("ExecutionFailed"),
                    "under-funded untrusted sub should revert ExecutionFailed, got: {outcome}"
                );
            }
        }

        Ok(())
    }

    /// On-chain redeemable periods = isValidOrRedeemable(id) / amount.
    async fn onchain_periods<P: Provider>(
        provider: &P,
        sub: &RedeemableSubscription,
    ) -> Result<i32> {
        let module = sub.contract_address;
        let id = sub.id;
        let redeemable = SubView::new(module, provider)
            .isValidOrRedeemable(id)
            .call()
            .await?;
        let amount = U256::from_str_radix(&sub.amount, 10)?;
        Ok((redeemable / amount).to::<u64>() as i32)
    }

    /// LIVE: actually broadcasts one `redeem` tx per subscription, sequentially.
    /// Uses `redeem_singular` (direct single tx) — NOT the EIP-7702 batch — so a
    /// reverting sub can't take down the others. Requires `REDEEMER_PK` in env/.env.
    #[tokio::test]
    #[ignore = "LIVE: broadcasts real txs & spends gas; requires REDEEMER_PK; run with --ignored --nocapture"]
    async fn redeem_each_individually_live() -> Result<()> {
        dotenv::dotenv().ok();
        let pk = match std::env::var("REDEEMER_PK") {
            Ok(pk) => pk,
            Err(_) => {
                eprintln!("REDEEMER_PK not set — skipping live redeem test");
                return Ok(());
            }
        };
        let signer = PrivateKeySigner::from_str(pk.trim())?;
        println!("\nLive redeem (one at a time) from {}", signer.address());
        println!("{:-<100}", "");

        let provider = ProviderBuilder::new().connect_http(GNOSIS_RPC.parse()?);
        for mut sub in fixture() {
            sub.periods = onchain_periods(&provider, &sub).await?;
            let tag = format!(
                "{:<10} {}",
                format!("{:?}", sub.category),
                &format!("{:#x}", sub.id)[..10]
            );
            match redeem_singular(GNOSIS_RPC, signer.clone(), &sub).await {
                Ok(hash) => println!("{tag}  ✅ https://gnosisscan.io/tx/{hash}"),
                Err(e) => println!("{tag}  ❌ {e}"),
            }
        }
        Ok(())
    }
}
