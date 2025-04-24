# Metis Tools

## Blockfetch Tool

`metis-blockfetch` is a tool for fetching blocks from an RPC provider, and snapshot that block data to the folder `data/blocks` with the JSON format.

```shell
cargo run -r --bin metis-blockfetch <RPC_URL> <BLOCK_ID>
```

Where `<BLOCK_ID>` is a block hash or a block number.

For example

```shell
cargo run -r --bin metis-blockfetch https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27 10889447
```

## Ethertest Tool

`metis-ethertest` is a tool for running [ethertest](https://github.com/ethereum/tests).

### State Tests

Prepare Test Suites

```shell
git clone https://github.com/ethereum/tests
```

Run the following command to test

```shell
cargo run -r --bin metis-ethertest run tests/GeneralStateTests
```

## Blocktest Tool

`metis-blocktest` is tool for running real Ethereum block transactions.

Run the following command to test

```shell
cargo run -r --bin metis-blocktest run data/blocks
```
