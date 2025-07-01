use std::{collections::BTreeMap, fs::File, sync::Arc, time::Instant};

use alloy_consensus::Transaction as _;
use alloy_provider::{Provider, ProviderBuilder, network::primitives::BlockTransactions};
use alloy_rpc_types_eth::{BlockId, transaction::Transaction};
use anyhow::{Context as _, Result, anyhow};
use clap::Parser;
use indexmap::IndexMap;
use indicatif::ProgressBar;
use metis_primitives::{
    AccessListItem, Address, B256, Bytes, HashMap, SpecId, Transaction as _, TxEnv, TxKind, U256,
};
use metis_tools::get_block_spec;
use reqwest::Url;
use revm::context::ContextSetters;
use revm::{
    Context, ExecuteCommitEvm, MainBuilder, MainContext,
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

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
struct TestEnv {
    pub block_number: u64,
    pub current_coinbase: Address,
    pub current_difficulty: U256,
    pub current_gas_limit: U256,
    pub current_timestamp: U256,
    pub previous_hash: B256,
    pub base_fee_per_gas: Option<U256>,
    pub mix_hash: Option<B256>,
}

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, Clone)]
struct TestTransaction {
    pub data: Bytes,
    pub gas_limit: U256,
    pub gas_price: Option<U256>,
    pub nonce: U256,
    #[serde(skip)]
    pub secret_key: Option<B256>,
    #[serde(default)]
    pub sender: Option<Address>,
    pub to: Option<Address>,
    pub value: U256,
    pub max_fee_per_gas: Option<U256>,
    pub max_priority_fee_per_gas: Option<U256>,
    pub max_fee_per_blob_gas: Option<U256>,
    pub access_list: Option<Vec<AccessListItem>>,
}

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExpectLog {
    pub address: Address,
    pub data: Bytes,
    pub topics: Vec<B256>,
    pub block_hash: Option<B256>,
    pub block_number: Option<U256>,
    pub transaction_hash: Option<B256>,
    pub transaction_index: Option<U256>,
    pub log_index: Option<U256>,
    pub removed: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    pub balance: U256,
    pub code: Bytes,
    pub code_hash: Option<B256>,
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
    let provider = ProviderBuilder::new().connect_http(rpc_url);
    let block = provider
        .get_block(block_id)
        .full()
        .await
        .with_context(|| "Failed to get block")?
        .ok_or_else(|| anyhow!("No block found for ID: {block_id:?}"))?;
    let spec_id = get_block_spec(block.header.timestamp, block.header.number);
    let previous_block_number = block.header.number.saturating_sub(1);
    // Use the previous block state as the db with caching
    let prev_id: BlockId = previous_block_number.into();
    let state_db = WrapDatabaseAsync::new(AlloyDB::new(provider.clone(), prev_id)).unwrap();
    let cache_db: CacheDB<_> = CacheDB::new(state_db);
    let mut state = StateBuilder::new_with_database(cache_db).build();
    state.set_state_clear_flag(spec_id.is_enabled_in(SpecId::SPURIOUS_DRAGON));
    let ctx = Context::mainnet()
        .with_db(&mut state)
        .modify_block_chained(|b| {
            b.number = U256::from(block.header.number);
            b.beneficiary = block.header.beneficiary;
            b.timestamp = U256::from(block.header.timestamp);
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
    println!(
        "Found {txs} transactions in the block {}",
        block.header.number
    );
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
                let res = evm.replay_commit()?;
                logs.push(res.logs().to_vec());
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
                let res = evm.replay_commit()?;
                assert_eq!(res.gas_used(), receipt.gas_used);
                logs.push(res.logs().to_vec());
                console_bar.inc(1);
            }
        }
        BlockTransactions::Uncle => panic!("Wrong transaction type"),
    }
    let elapsed = start.elapsed();
    println!(
        "Finished execution. Total CPU time: {:.6}s",
        elapsed.as_secs_f64()
    );
    let block_file = File::create(format!("data/blocks/{}.json", block.header.number))?;
    // The key denotes the tx index + 1
    let transactions: IndexMap<String, TestTransaction> = txs
        .iter()
        .enumerate()
        .map(|(idx, tx)| {
            (
                (idx + 1).to_string(),
                TestTransaction {
                    data: tx.data.clone(),
                    gas_limit: U256::from(tx.gas_limit),
                    gas_price: Some(U256::from(tx.gas_price)),
                    nonce: U256::from(tx.nonce),
                    secret_key: None,
                    sender: Some(tx.caller),
                    to: tx.kind.to().cloned(),
                    value: tx.value,
                    max_fee_per_gas: Some(U256::from(tx.max_fee_per_gas())),
                    max_priority_fee_per_gas: tx.max_priority_fee_per_gas().map(|v| U256::from(v)),
                    max_fee_per_blob_gas: Some(U256::from(tx.max_fee_per_blob_gas())),
                    access_list: Some(tx.access_list.to_vec()),
                },
            )
        })
        .collect();
    // The key denotes the tx index
    let logs: IndexMap<String, HashMap<String, ExpectLog>> = logs
        .iter()
        .enumerate()
        .map(|(idx, logs)| {
            (
                idx.to_string(),
                logs.iter()
                    .enumerate()
                    .map(|(idx, log)| {
                        (
                            idx.to_string(),
                            ExpectLog {
                                address: log.address,
                                data: log.data.data.clone(),
                                topics: log.topics().to_vec(),
                                block_hash: Some(block.header.hash),
                                block_number: Some(U256::from(block.header.number)),
                                transaction_index: Some(U256::from(idx)),
                                log_index: Some(U256::from(idx)),
                                removed: Some(false),
                                ..Default::default()
                            },
                        )
                    })
                    .collect(),
            )
        })
        .collect();
    let mut block_hashes: IndexMap<String, B256> = Default::default();
    println!(
        "Fetching previous 256 block hashes from {} to {}...",
        block.header.number.saturating_sub(256),
        block.header.number
    );
    let console_bar = Arc::new(ProgressBar::new(256));
    for number in block.header.number.saturating_sub(256)..block.header.number {
        block_hashes.insert(
            number.to_string(),
            provider
                .get_block(block_id)
                .full()
                .await
                .with_context(|| "Failed to get block")?
                .ok_or_else(|| anyhow!("No block found for ID: {block_id:?}"))?
                .header
                .hash,
        );
        console_bar.inc(1);
    }
    println!("Fetching pre state...");
    let mut pre_state = BTreeMap::<Address, TestAccount>::new();
    for (address, account) in state.cache.trie_account() {
        let code = provider
            .get_code_at(address)
            .block_id(previous_block_number.into())
            .await?;
        let mut storage: HashMap<U256, U256> = Default::default();
        for k in account.storage.keys() {
            storage.insert(
                *k,
                provider
                    .get_storage_at(address, *k)
                    .block_id(previous_block_number.into())
                    .await?,
            );
        }
        if let Ok(trie_account) = provider
            .get_account(address)
            .block_id(previous_block_number.into())
            .await
        {
            pre_state.insert(
                address,
                TestAccount {
                    balance: trie_account.balance,
                    code_hash: trie_account.code_hash,
                    code: Some(code),
                    nonce: trie_account.nonce,
                    storage,
                },
            );
        }
    }
    let json_suite = json!({
        "test_name": {
            "env": TestEnv {
                block_number: block.header.number,
                current_coinbase: block.header.beneficiary,
                current_gas_limit: U256::from(block.header.gas_limit),
                current_difficulty: block.header.difficulty,
                current_timestamp: U256::from(block.header.timestamp),
                previous_hash: block.header.parent_hash,
                base_fee_per_gas: block.header.base_fee_per_gas.map(|v|U256::from(v)),
                mix_hash: Some(block.header.mix_hash),
            },
            "transactions": transactions,
            "logs": logs,
            "block_hashes": block_hashes,
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
