//! Simplified batch execution for parallel transaction processing.
//!
//! This module provides the core parallel execution logic that integrates
//! metis-pe with reth's payload building, following the chatGPT combined approach.
//!
//! Key principle: Execute transactions once in parallel via metis-pe, then
//! commit results to builder. No cache, no double execution.

use alloy_evm::EvmEnv;
use metis_pe::{ParallelExecutor, TxExecutionResult};
use metis_primitives::TxEnv;
use std::num::NonZeroUsize;

/// Execute transactions in parallel using metis-pe and return complete results.
///
/// This function:
/// 1. Executes all transactions in parallel using Block-STM
/// 2. Returns Vec<TxExecutionResult> containing receipts and ResultAndState
///
/// The caller is responsible for committing these results to the builder.
///
/// # Arguments
/// * `db` - Database adapter for state access
/// * `evm_env` - EVM environment (block env + config env)
/// * `tx_envs` - Transaction environments to execute
/// * `threads` - Number of parallel threads
///
/// # Returns
/// Vec<TxExecutionResult> with complete execution results including ResultAndState
pub fn execute_parallel_batch<DB>(
    db: DB,
    evm_env: EvmEnv,
    tx_envs: Vec<TxEnv>,
    threads: NonZeroUsize,
) -> Result<Vec<TxExecutionResult<metis_primitives::HaltReason>>, String>
where
    DB: metis_primitives::DatabaseRef + Send + Sync,
{
    // Explicit HR to avoid inference ambiguity after HR was made generic in metis-pe.
    let mut executor: ParallelExecutor<metis_primitives::HaltReason> = ParallelExecutor::default();

    executor
        .execute(db, evm_env, tx_envs, threads)
        .map_err(|e| format!("Parallel execution failed: {:?}", e))
}

/// Determine if parallel execution should be used based on transaction count.
///
/// # Arguments
/// * `tx_count` - Number of transactions in the block
/// * `threshold` - Minimum transactions for parallel execution (default: 2)
///
/// # Returns
/// true if parallel execution should be used
pub fn should_use_parallel(tx_count: usize, threshold: usize) -> bool {
    tx_count >= threshold
}

/// Get optimal thread count for parallel execution.
///
/// Returns the number of available CPU cores, with a minimum of 1.
pub fn get_thread_count() -> NonZeroUsize {
    NonZeroUsize::new(num_cpus::get()).unwrap_or(NonZeroUsize::MIN)
}
