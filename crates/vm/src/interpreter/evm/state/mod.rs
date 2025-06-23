use crate::interpreter::evm::store::block::ReadOnlyBlockstore;
use exec::EvmExecState;
use std::sync::Arc;

pub mod exec;
pub mod genesis;
pub mod query;

pub type CheckStateRef<DB> = Arc<tokio::sync::Mutex<Option<EvmExecState<ReadOnlyBlockstore<DB>>>>>;
