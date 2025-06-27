use crate::interpreter::evm::{
    CheckInterpreter, EvmMessageInterpreter,
    state::exec::EvmExecState,
    store::block::{Blockstore, ReadOnlyBlockstore},
};
use async_trait::async_trait;

use metis_primitives::{Address, Env, InvalidTransaction};

/// Transaction check results are expressed by the exit code, so that hopefully
/// they would result in the same error code if they were applied.
pub struct EvmCheckRet {
    pub sender: Address,
    pub gas_limit: u64,
    pub error: Option<InvalidTransaction>,
    pub info: Option<String>,
}

#[async_trait]
impl<DB, TC> CheckInterpreter for EvmMessageInterpreter<DB, TC>
where
    DB: Blockstore + 'static + Send + Sync,
    TC: Send + Sync + 'static,
{
    // We simulate the full pending state so that client can call methods on
    // contracts that haven't been deployed yet.
    // TODO: change it to the state db.
    type State = EvmExecState<ReadOnlyBlockstore<DB>>;
    type Message = Env;
    type Output = EvmCheckRet;

    /// Check that:
    /// * sender exists
    /// * sender nonce matches the message sequence
    /// * sender has enough funds to cover the gas cost
    async fn check(
        &self,
        _state: Self::State,
        msg: Self::Message,
        _is_recheck: bool,
    ) -> anyhow::Result<Self::Output> {
        Ok(EvmCheckRet {
            sender: msg.tx.caller,
            gas_limit: msg.tx.gas_limit,
            error: None,
            info: None,
        })
    }
}
