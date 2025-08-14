use alloy_evm::block::BlockExecutionResult;
use metis_chain::provider::ParallelEthEvmConfig;
use reth_evm::execute::BasicBlockExecutor;
use reth_evm::execute::Executor;
use std::error::Error;

pub mod common;

#[tokio::test]
async fn test_withdraw() -> Result<(), Box<dyn Error>> {
    let (keypair, sender) = common::keypair::get_random_keypair();
    let (chain_spec, db, recovered_block) = common::tx::get_test_withdraw_config(sender, keypair);
    let mut executor = BasicBlockExecutor::new(ParallelEthEvmConfig::new(chain_spec), db);

    let BlockExecutionResult {
        receipts, requests, ..
    } = executor.execute_one(&recovered_block).unwrap();

    let receipt = receipts.first().unwrap();

    println!("receipt {receipt:?}");
    assert!(receipt.success);

    // There should be exactly one entry with withdrawal requests
    println!("request {requests:?}");
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0][0], 1);

    Ok(())
}
