#!/usr/bin/env bash
#
#
# start 3 node chain for local test
# include: 
#   3 metis nodes
#   3 malaketh-layered nodes
#


# start metis nodes
# as reth engine
docker compose up -d

# init chain config
cp -fr nodes_config_bin nodes

# Establishing links between peers
#./add_peers.sh

# waiting for metis node completely started
sleep 3

# start malaketh-layered nodes
# as consensus components
docker compose -f compose-mala.yaml up -d
