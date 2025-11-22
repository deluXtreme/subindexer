use alloy::{
    eips::eip7702::Authorization,
    network::{TransactionBuilder, TransactionBuilder7702},
    primitives::{
        Address, Bytes, TxHash, TxKind, U256, address,
        bytes::{BufMut, BytesMut},
    },
    providers::{Provider, ProviderBuilder},
    rpc::types::{BlockId, TransactionRequest},
    signers::{SignerSync, local::PrivateKeySigner},
    sol,
};
use anyhow::{Result, anyhow};

sol!(
    #[sol(rpc)]
    interface EOAMultisend {
        function execute(bytes calldata calls) external;
        function execute(bytes calldata calls, bytes calldata signature) external;
    }
);

const EOA_MULTISEND_ADDRESS: Address = address!("0x7702e0abc94b08eb155795c72624f2ca53763114");

pub async fn eoa_multisend(
    rpc_url: &str,
    pk: PrivateKeySigner,
    calls: Vec<TransactionRequest>,
) -> Result<TxHash> {
    let provider = ProviderBuilder::new()
        .wallet(pk.clone())
        .connect_http(rpc_url.parse()?);

    let contract = EOAMultisend::new(EOA_MULTISEND_ADDRESS, provider.clone());

    let authorization = Authorization {
        chain_id: U256::from(provider.get_chain_id().await?),
        address: *contract.address(),
        nonce: provider
            .get_transaction_count(pk.address())
            .block_id(BlockId::pending())
            .await?
            + 1,
    };

    let signature = pk.sign_hash_sync(&authorization.signature_hash())?;
    let signed_authorization = authorization.into_signed(signature);

    let batched = pack_multisend(&calls)?;

    let call = contract.execute_0(batched);
    let execute_calldata = call.calldata().to_owned();

    let tx = TransactionRequest::default()
        .with_to(pk.address())
        .with_authorization_list(vec![signed_authorization])
        .with_input(execute_calldata);

    // Send the transaction and wait for the broadcast.
    let receipt = provider.send_transaction(tx).await?.get_receipt().await?;

    Ok(receipt.transaction_hash)
}

/// Encode a list of EOA-style txs for MultiSend/MultiSendCallOnly.
fn pack_multisend(calls: &[TransactionRequest]) -> Result<Bytes> {
    let mut out = BytesMut::new();

    for tx in calls {
        // operation ─ normal CALL
        out.put_u8(0);

        // to ─ 20 bytes (error if missing)
        let to = match tx.to.as_ref() {
            Some(TxKind::Call(addr)) => addr,
            _ => return Err(anyhow!("Multisend: tx missing `to`")),
        };
        out.extend_from_slice(to.as_slice());

        // value ─ 32 bytes big-endian
        let value: U256 = tx.value.unwrap_or_default();
        out.extend_from_slice(&value.to_be_bytes::<32>());

        // data length & data
        let data: Bytes = tx
            .input
            .input // first try the “input” field
            .clone()
            .or_else(|| tx.input.data.clone()) // else fall back to “data”
            .unwrap_or_default(); // or empty
        out.extend_from_slice(&U256::from(data.len()).to_be_bytes::<32>());
        out.extend_from_slice(&data);
    }

    Ok(Bytes::from(out.freeze()))
}
