# Metis Tools

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
