#!/usr/bin/env bash

# Script to manually add peers (their enodes) to each node

# Configuration
RPC_BASE_URL="http://127.0.0.1"
RPC_PORTS=(8545 18545 28545)

sleep 5

# Retrieve node ENode addresses
RETH0_ENODE=$(cast rpc --rpc-url "${RPC_BASE_URL}:${RPC_PORTS[0]}" admin_nodeInfo | jq -r .enode)
RETH1_ENODE=$(cast rpc --rpc-url "${RPC_BASE_URL}:${RPC_PORTS[1]}" admin_nodeInfo | jq -r .enode)
RETH2_ENODE=$(cast rpc --rpc-url "${RPC_BASE_URL}:${RPC_PORTS[2]}" admin_nodeInfo | jq -r .enode)

# Configure reth0 to connect with reth1 and reth2
echo "RETH0_ENODE: ${RETH0_ENODE}"
cast rpc --rpc-url "${RPC_BASE_URL}:${RPC_PORTS[0]}" admin_addTrustedPeer "${RETH1_ENODE}"
cast rpc --rpc-url "${RPC_BASE_URL}:${RPC_PORTS[0]}" admin_addTrustedPeer "${RETH2_ENODE}"
cast rpc --rpc-url "${RPC_BASE_URL}:${RPC_PORTS[0]}" admin_addPeer "${RETH1_ENODE}"
cast rpc --rpc-url "${RPC_BASE_URL}:${RPC_PORTS[0]}" admin_addPeer "${RETH2_ENODE}"

# Configure reth1 to connect with reth0 and reth2
echo "RETH1_ENODE: ${RETH1_ENODE}"
cast rpc --rpc-url "${RPC_BASE_URL}:${RPC_PORTS[1]}" admin_addTrustedPeer "${RETH0_ENODE}"
cast rpc --rpc-url "${RPC_BASE_URL}:${RPC_PORTS[1]}" admin_addTrustedPeer "${RETH2_ENODE}"
cast rpc --rpc-url "${RPC_BASE_URL}:${RPC_PORTS[1]}" admin_addPeer "${RETH0_ENODE}"
cast rpc --rpc-url "${RPC_BASE_URL}:${RPC_PORTS[1]}" admin_addPeer "${RETH2_ENODE}"

# Configure reth2 to connect with reth0 and reth1
echo "RETH2_ENODE: ${RETH2_ENODE}"
cast rpc --rpc-url "${RPC_BASE_URL}:${RPC_PORTS[2]}" admin_addTrustedPeer "${RETH0_ENODE}"
cast rpc --rpc-url "${RPC_BASE_URL}:${RPC_PORTS[2]}" admin_addTrustedPeer "${RETH1_ENODE}"
cast rpc --rpc-url "${RPC_BASE_URL}:${RPC_PORTS[2]}" admin_addPeer "${RETH0_ENODE}"
cast rpc --rpc-url "${RPC_BASE_URL}:${RPC_PORTS[2]}" admin_addPeer "${RETH1_ENODE}"
