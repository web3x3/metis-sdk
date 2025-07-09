#!/usr/bin/env bash
#
#
# start 3 node chain for local test
# include: 
#   3 metis nodes
#   3 malaketh-layered nodes
#

# create genesis data
cd ../lib/malaketh-layered
cargo run --bin malachitebft-eth-utils genesis
cd -

# start metis nodes
./metis_nodes.sh

./add_peers.sh

./malachite_nodes.sh
