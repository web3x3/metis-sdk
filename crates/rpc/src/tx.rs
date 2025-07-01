use async_trait::async_trait;
use bincode::serialize;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use metis_primitives::{Address, TxEnv, U256};
use tendermint::abci::types::ExecTxResult;
use tendermint_rpc::endpoint::broadcast::{tx_async, tx_commit, tx_sync};

#[derive(Clone, Debug)]
/// gas params
pub struct GasParams {
    /// Maximum amount of gas that can be charged.
    pub gas_limit: u64,
    /// Price of gas.
    ///
    /// Any discrepancy between this and the base fee is paid for
    /// by the validator who puts the transaction into the block.
    pub gas_fee_cap: U256,
    /// Gas premium.
    pub gas_premium: U256,
}

#[allow(missing_docs)]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedMessage {
    tx: TxEnv,
    signature: Vec<u8>,
}

impl SignedMessage {
    /// serialize for `SignedMessage`
    pub fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        Ok(serialize(&self)?)
    }
}

/// Abstracting away what the return value is based on whether
/// we broadcast transactions in sync, async or commit mode.
pub trait BroadcastMode {
    /// Response wrapper
    type Response<T>;
}

/// client for submitting transactions.
#[async_trait]
pub trait TxClient<M: BroadcastMode = TxCommit>: Send + Sync {
    /// Transfer tokens to another account.
    async fn transfer(
        &mut self,
        _to: Address,
        _value: U256,
        _gas_params: GasParams,
    ) -> anyhow::Result<M::Response<()>> {
        // todo make SignedMsg
        // let tx = MakeTx(from, to, amount)
        // let sig = Sign(tx)
        // let msg = SignedMessage{tx, sig}
        let msg = SignedMessage {
            tx: Default::default(),
            signature: vec![],
        };
        let fut = self.perform(msg, |_| Ok(()));
        let res = fut.await?;
        Ok(res)
    }

    #[allow(missing_docs)]
    async fn perform<F, T>(&self, msg: SignedMessage, f: F) -> anyhow::Result<M::Response<T>>
    where
        F: FnOnce(&ExecTxResult) -> anyhow::Result<T> + Sync + Send,
        T: Sync + Send;
}
/// Return immediately after the transaction is broadcasted without waiting for check results.
#[derive(Debug)]
pub struct TxAsync;
/// Wait for the check results before returning from broadcast.
#[derive(Debug)]
pub struct TxSync;
/// Wait for the delivery results before returning from broadcast.
#[derive(Debug)]
pub struct TxCommit;

#[derive(Debug)]
/// Response from Tendermint
pub struct AsyncResponse<T> {
    /// Response from Tendermint.
    pub response: tx_async::Response,
    /// Parsed return data, if the response indicates success.
    pub return_data: PhantomData<T>,
}

#[derive(Debug)]
/// Response from Tendermint
pub struct SyncResponse<T> {
    /// Response from Tendermint.
    pub response: tx_sync::Response,
    /// Parsed return data, if the response indicates success.
    pub return_data: PhantomData<T>,
}

#[derive(Debug)]
/// Response from Tendermint
pub struct CommitResponse<T> {
    /// Response from Tendermint.
    pub response: tx_commit::Response,
    /// Parsed return data, if the response indicates success.
    pub return_data: Option<T>,
}

impl BroadcastMode for TxAsync {
    type Response<T> = AsyncResponse<T>;
}

impl BroadcastMode for TxSync {
    type Response<T> = SyncResponse<T>;
}

impl BroadcastMode for TxCommit {
    type Response<T> = CommitResponse<T>;
}
