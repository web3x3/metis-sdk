use alloy_evm::block::BlockExecutionResult;
use metis_chain::provider::BlockParallelExecutorProvider;
use pretty_assertions::assert_eq;
use reth_evm::execute::{BasicBlockExecutorProvider, BlockExecutorProvider, Executor};
use reth_evm_ethereum::EthEvmConfig;
use std::error::Error;

pub mod common;

// #[tokio::test]
// TODO: https://github.com/MetisProtocol/metis-sdk/issues/84
async fn _test_compare_receipt() -> Result<(), Box<dyn Error>> {
    let (keypair, sender) = common::keypair::get_random_keypair();
    println!("Sending tx: {:?}", sender);

    let parallel_receipt = {
        let (chain_spec, db, recovered_block) =
            common::tx::get_test_withdraw_config(sender, keypair);
        let provider = BlockParallelExecutorProvider::new(EthEvmConfig::new(chain_spec));
        let mut executor = provider.executor(db);

        let BlockExecutionResult { receipts, .. } = executor.execute_one(&recovered_block).unwrap();
        receipts.first().unwrap().clone()
    };

    let sequential_receipt = {
        let (chain_spec, db, recovered_block) =
            common::tx::get_test_withdraw_config(sender, keypair);
        let provider = BasicBlockExecutorProvider::new(EthEvmConfig::new(chain_spec));
        let mut executor = provider.executor(db);
        let BlockExecutionResult { receipts, .. } = executor.execute_one(&recovered_block).unwrap();
        receipts.first().unwrap().clone()
    };

    assert_eq!(parallel_receipt, sequential_receipt);

    Ok(())
}
