use fvm_ipld_blockstore::Blockstore;
use serde::{Deserialize, Serialize};
use fvm::{
    call_manager::DefaultCallManager,
    executor::{DefaultExecutor},
    machine::{DefaultMachine},
    DefaultKernel,
};
use crate::interpreter::store::externs::FendermintExterns;

/// Parts of the state which evolve during the lifetime of the chain.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct EvmStateParams {
    // pub state_root: Cid,
    // pub timestamp: Timestamp,
    // pub network_version: NetworkVersion,
    // pub base_fee: TokenAmount,
    // pub circ_supply: TokenAmount,
    // /// The [`ChainID`] is stored here to hint at the possibility that
    // /// a chain ID might change during the lifetime of a chain, in case
    // /// there is a fork, or perhaps a subnet migration in IPC.
    // ///
    // /// How exactly that would be communicated is uknown at this point.
    // pub chain_id: u64,
}

/// A state we create for the execution of all the messages in a block.
#[allow(dead_code)]
pub struct EvmExecState<DB>
where
    DB: Blockstore + 'static, {
    executor:
        DefaultExecutor<DefaultKernel<DefaultCallManager<DefaultMachine<DB, FendermintExterns>>>>,
}
