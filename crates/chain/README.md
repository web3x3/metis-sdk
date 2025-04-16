# Chain

This document provides a guide on utilizing the Metis SDK to rapidly construct blockchain nodes. The implementation leverages parallel transaction execution to enhance performance while maintaining compatibility with Ethereum's execution layer standards.

## Quick Start

Execute the following command to launch a Metis devnet:

```shell
cargo run -r --bin metis --dev
```

## Core Components

### `ParallelExecutorBuilder`

The `ParallelExecutorBuilder` implements the `ExecutorBuilder` trait, serving as a factory for creating parallelized EVM executors.

```rust
#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct ParallelExecutorBuilder;

impl ExecutorBuilder for ParallelExecutorBuilder {
    async fn build_evm(
        self,
        ctx: &BuilderContext<Node>,
    ) -> eyre::Result<(Self::EVM, Self::Executor)> {
        // Implementation details
    }
}
```

### `ParallelExecutor`

During specific execution, the parallel executor needs to process 3 stages of block content:

- Applies any necessary changes before executing the block's transactions.
- Executing block transactions
- Applies any necessary changes after executing the block's transactions, completes execution and returns the underlying EVM along with execution result.

```rust
pub struct ParallelExecutor<DB> {
    strategy_factory: EthEvmConfig,
    db: State<DB>,
    executor: metis_pe::ParallelExecutor,
    concurrency_level: NonZeroUsize,
}

// perform 3 steps to complete the block calculation
fn execute_one(
    &mut self,
    block: &RecoveredBlock<<<Self as Executor<DB>>::Primitives as NodePrimitives>::Block>,
) -> Result<BlockExecutionResult<Receipt>, BlockExecutionError> {}

// parallel execute block transactions which belongs to the second stage
pub fn execute_block(
    &mut self,
    block: &RecoveredBlock<<<Self as Executor<DB>>::Primitives as NodePrimitives>::Block>,
) -> Result<BlockExecutionResult<Receipt>, BlockExecutionError> {}
```

### NodeBuilder

We use general `reth::builder::NodeBuilder<ChainSpec>` to build chain nodes. `NodeBuilder` provides execution plugins. The advantage of this is that the metis-sdk provider can be run in a specific node as a plugin.

```rust
pub struct NodeBuilder<DB, ChainSpec> {
    /// All settings for how the node should be configured.
    config: NodeConfig<ChainSpec>,
    /// The configured database for the node.
    database: DB,
}
```

The `ChainSpec` comes from genesis file, for example:

```rust
pub fn custom_chain() -> Arc<ChainSpec> {
    let custom_genesis = r#"
{
    "nonce": "0x42",
    "timestamp": "0x0",
    "extraData": "0x5343",
    "gasLimit": "0x5208",
    "difficulty": "0x400000000",
    "mixHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
    "coinbase": "0x0000000000000000000000000000000000000000",
    "alloc": {
        "0x6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b": {
            "balance": "0x4a47e3c12448f4ad000000"
        }
    },
    "number": "0x0",
    "gasUsed": "0x0",
    "parentHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
    "config": {
        "ethash": {},
        "chainId": 2600,
        "homesteadBlock": 0,
        "eip150Block": 0,
        "eip155Block": 0,
        "eip158Block": 0,
        "byzantiumBlock": 0,
        "constantinopleBlock": 0,
        "petersburgBlock": 0,
        "istanbulBlock": 0,
        "berlinBlock": 0,
        "londonBlock": 0,
        "terminalTotalDifficulty": 0,
        "terminalTotalDifficultyPassed": true,
        "shanghaiTime": 0
    }
}
"#;
    let genesis: Genesis = serde_json::from_str(custom_genesis).unwrap();
    Arc::new(genesis.into())
}
```

We can also use `reth_ethereum::chainspec::ChainSpec` dependency, which has the general genesis configuration of various chains:

```rust
pub static MAINNET: LazyLock<Arc<ChainSpec>> = LazyLock::new(|| {..});
pub static SEPOLIA: LazyLock<Arc<ChainSpec>> = LazyLock::new(|| {..});
pub static HOLESKY: LazyLock<Arc<ChainSpec>> = LazyLock::new(|| {..});
...
```

## Example

Here is an example for how to build an node with the `NodeBuilder`.

```rust
pub fn get_test_node_config() -> NodeConfig<ChainSpec> {
    NodeConfig::test()
        .dev()
        .with_rpc(RpcServerArgs::default().with_http())
        .with_chain(custom_chain())
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let node_config = get_test_node_config();

    let NodeHandle {
        node,
        node_exit_future: _,
    } = NodeBuilder::new(node_config)
        .testing_node(tasks.executor())
        .with_types::<EthereumNode>()
        .with_components(
            // Note: we use `ParallelExecutorBuilder` here.
            EthereumNode::components().executor(ParallelExecutorBuilder::default()),
        )
        .with_add_ons(EthereumAddOns::default())
        .launch()
        .await
        .unwrap();

    node_exit_future.await
}
```
