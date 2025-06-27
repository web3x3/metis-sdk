use crate::interpreter::evm::store::block::Blockstore;
use metis_primitives::B256;
use std::sync::Arc;

/// Parts of the state which evolve during the lifetime of the chain.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct EvmStateParams {
    pub state_root: B256,
}

/// A state we create for the execution of all the messages in a block.
#[allow(dead_code)]
pub struct EvmExecState<DB> {
    pub db: Arc<DB>,
}

impl<DB> EvmExecState<DB>
where
    DB: Blockstore + 'static,
{
    pub fn new(// blockstore: DB,
        // multi_engine: &MultiEngine,
        // block_height: ChainEpoch,
        // params: FvmStateParams,
    ) -> anyhow::Result<Self> {
        todo!()
    }
}
