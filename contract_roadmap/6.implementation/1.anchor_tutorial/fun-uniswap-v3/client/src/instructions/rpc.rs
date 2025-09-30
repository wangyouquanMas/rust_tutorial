use anyhow::{anyhow, Result};
use solana_client::{
    rpc_client::RpcClient,
    rpc_config::RpcSendTransactionConfig,
    rpc_request::RpcRequest,
    rpc_response::{RpcResult, RpcSimulateTransactionResult},
};

use solana_sdk::{
    commitment_config::CommitmentConfig,transaction::Transaction, signature::Signature,
};

pub fn send_txn(client: &RpcClient, txn: &Transaction, wait_confirm: bool) -> Result<Signature> {
    Ok(client.send_and_confirm_transaction_with_spinner_and_config(
        txn,
        if wait_confirm {
            CommitmentConfig::confirmed()
        } else {
            CommitmentConfig::processed()
        },
        RpcSendTransactionConfig {
            skip_preflight: true,
            ..RpcSendTransactionConfig::default()
        },
    )?)
}


pub fn simulate_transaction(
    client: &RpcClient,
    transaction: &Transaction,
    sig_verify: bool,
    cfg: CommitmentConfig,
) -> RpcResult<RpcSimulateTransactionResult> {
    let serialized_encoded = bs58::encode(bincode::serialize(transaction).unwrap()).into_string();
    client.send(
        RpcRequest::SimulateTransaction,
        serde_json::json!([serialized_encoded, {
            "sigVerify": sig_verify, "commitment": cfg.commitment
        }]),
    )
}