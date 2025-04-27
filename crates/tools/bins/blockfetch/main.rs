use std::{collections::BTreeMap, fs::File, sync::Arc, time::Instant};

use alloy_consensus::Transaction as _;
use alloy_provider::{Provider, ProviderBuilder, network::primitives::BlockTransactions};
use alloy_rpc_types_eth::{BlockId, transaction::Transaction};
use anyhow::{Context as _, Result, anyhow};
use clap::Parser;
use hashbrown::HashMap;
use indicatif::ProgressBar;
use metis_primitives::{Address, B256, Bytes, SpecId, TxEnv, TxKind, U256};
use metis_tools::get_block_spec;
use reqwest::Url;
use revm::{
    Context, ExecuteEvm, InspectEvm, MainBuilder, MainContext,
    database::{AlloyDB, CacheDB, StateBuilder, WrapDatabaseAsync},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct TestAccount {
    pub balance: U256,
    pub code: Option<Bytes>,
    pub code_hash: B256,
    pub nonce: u64,
    pub storage: HashMap<U256, U256>,
}

#[derive(Parser)]
#[command(author, version, about)]
#[command(propagate_version = true)]
struct Cli {
    rpc_url: Url,
    block_id: BlockId,
}

#[tokio::main]
async fn main() -> Result<()> {
    let Cli { rpc_url, block_id } = Cli::parse();
    let provider = ProviderBuilder::new().on_http(rpc_url);
    let block = provider
        .get_block(block_id)
        .await
        .with_context(|| "Failed to get block")?
        .ok_or_else(|| anyhow!("No block found for ID: {block_id:?}"))?;
    let spec_id = get_block_spec(block.header.timestamp, block.header.number);
    let previous_block_number = block.header.number - 1;
    // Use the previous block state as the db with caching
    let prev_id: BlockId = previous_block_number.into();
    let state_db = WrapDatabaseAsync::new(AlloyDB::new(provider.clone(), prev_id)).unwrap();
    let cache_db: CacheDB<_> = CacheDB::new(state_db);
    let mut state = StateBuilder::new_with_database(cache_db).build();
    state.set_state_clear_flag(spec_id.is_enabled_in(SpecId::SPURIOUS_DRAGON));
    let ctx = Context::mainnet()
        .with_db(&mut state)
        .modify_block_chained(|b| {
            b.number = block.header.number;
            b.beneficiary = block.header.beneficiary;
            b.timestamp = block.header.timestamp;
            b.difficulty = block.header.difficulty;
            b.gas_limit = block.header.gas_limit;
            b.basefee = block.header.base_fee_per_gas.unwrap_or_default();
        })
        .modify_cfg_chained(|c| {
            c.chain_id = 1;
            c.spec = spec_id;
        });
    let mut evm =
        ctx.build_mainnet_with_inspector(revm::inspector::inspectors::TracerEip3155::new_stdout());
    let txs = block.transactions.len();
    println!("Found {txs} transactions.");
    let console_bar = Arc::new(ProgressBar::new(txs as u64));
    let start = Instant::now();
    let mut logs = Vec::new();
    let mut txs = Vec::new();
    match &block.transactions {
        BlockTransactions::Full(transactions) => {
            for tx in transactions {
                let tx = tx_into_env(tx);
                txs.push(tx.clone());
                evm.set_tx(tx);
                let res = evm.replay()?;
                logs.push(res.result.logs().to_vec());
                console_bar.inc(1);
            }
        }
        BlockTransactions::Hashes(items) => {
            for item in items {
                let tx = provider.get_transaction_by_hash(*item).await?.unwrap();
                let receipt = provider
                    .get_transaction_receipt(*item)
                    .await?
                    .ok_or_else(|| anyhow!("No receipt found for transaction: {item:?}"))?;
                let tx = tx_into_env(&tx);
                txs.push(tx.clone());
                evm.set_tx(tx);
                let res = evm.inspect_replay()?;
                assert_eq!(res.result.gas_used(), receipt.gas_used);
                logs.push(res.result.logs().to_vec());
                console_bar.inc(1);
            }
        }
        BlockTransactions::Uncle => panic!("Wrong transaction type"),
    }
    let mut pre_state = BTreeMap::<Address, TestAccount>::new();
    for (address, account) in state.cache.trie_account() {
        pre_state.insert(
            address,
            TestAccount {
                balance: account.info.balance,
                code: account.info.code.as_ref().map(|code| code.original_bytes()),
                code_hash: account.info.code_hash,
                nonce: account.info.nonce,
                storage: account.storage.iter().map(|(k, v)| (*k, *v)).collect(),
            },
        );
    }
    let elapsed = start.elapsed();
    println!(
        "Finished execution. Total CPU time: {:.6}s",
        elapsed.as_secs_f64()
    );
    let block_file = File::create(format!("data/blocks/{}.json", block.header.number))?;
    let json_suite = json!({
        "test_name": {
            "env": &block,
            "transactions": txs,
            "logs": logs,
            "pre": pre_state,
        },
    });
    serde_json::to_writer_pretty(block_file, &json_suite)?;
    Ok(())
}

fn tx_into_env(tx: &Transaction) -> TxEnv {
    let mut etx = TxEnv {
        caller: tx.inner.signer(),
        gas_limit: tx.gas_limit(),
        gas_price: tx.gas_price().unwrap_or(tx.inner.max_fee_per_gas()),
        value: tx.value(),
        data: tx.input().to_owned(),
        gas_priority_fee: tx.max_priority_fee_per_gas(),
        chain_id: Some(1),
        nonce: tx.nonce(),
        access_list: tx.access_list().cloned().unwrap_or_default(),
        kind: match tx.to() {
            Some(to_address) => TxKind::Call(to_address),
            None => TxKind::Create,
        },
        ..Default::default()
    };
    let _ = etx.derive_tx_type();
    etx
}
