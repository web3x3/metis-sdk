use alloy_evm::block::BlockExecutionResult;
use metis_chain::provider::ParallelEthEvmConfig;
use pretty_assertions::assert_eq;
use reth_evm::execute::{BasicBlockExecutor, Executor};
use reth_evm_ethereum::EthEvmConfig;
use std::error::Error;

pub mod common;

#[tokio::test]
async fn test_compare_receipt() -> Result<(), Box<dyn Error>> {
    let (keypair, sender) = common::keypair::get_random_keypair();
    println!("Sending tx: {sender:?}");

    let parallel_receipt = {
        let (chain_spec, db, recovered_block) =
            common::tx::get_test_withdraw_config(sender, keypair);
        let mut executor = BasicBlockExecutor::new(ParallelEthEvmConfig::new(chain_spec), db);

        let BlockExecutionResult { receipts, .. } = executor.execute_one(&recovered_block).unwrap();
        receipts.first().unwrap().clone()
    };

    let sequential_receipt = {
        let (chain_spec, db, recovered_block) =
            common::tx::get_test_withdraw_config(sender, keypair);

        let mut executor = BasicBlockExecutor::new(EthEvmConfig::new(chain_spec), db);

        let BlockExecutionResult { receipts, .. } = executor.execute_one(&recovered_block).unwrap();
        receipts.first().unwrap().clone()
    };

    assert_eq!(parallel_receipt, sequential_receipt);

    Ok(())
}
