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
# as reth engine
./metis_nodes.sh

# Establishing links between peers
./add_peers.sh

# start malaketh-layered nodes
# as consensus components
./malachite_nodes.sh
