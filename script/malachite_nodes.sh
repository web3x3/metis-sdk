#!/usr/bin/env bash

cd ../lib/malaketh-layered
cargo run --bin malachitebft-eth-app -- testnet --nodes 3 --home nodes
echo 👉 Grafana dashboard is available at http://localhost:3000
bash scripts/spawn.bash --nodes 3 --home nodes
cd -
