#!/usr/bin/env bash
set -euo pipefail  # Enable strict mode: exit immediately on errors

# Configuration constants (centralized management of reusable parameters)
BASE_DIR="./test"
GENESIS_PATH="../lib/malaketh-layered/assets/genesis.json"
JWT_SECRET="../lib/malaketh-layered/assets/jwtsecret"
LOG_DIR="${BASE_DIR}/logs"
VERBOSITY="-vvvvv"  # Log verbosity level
NODE_COUNT=3         # Number of nodes

# Initialize log directory
mkdir -p "${LOG_DIR}" || { echo "ERROR: Failed to create log directory ${LOG_DIR}"; exit 1; }

# Check if dependency files exist
if [ ! -f "${GENESIS_PATH}" ]; then
  echo "ERROR: Genesis file not found - ${GENESIS_PATH}"
  exit 1
fi

if [ ! -f "${JWT_SECRET}" ]; then
  echo "ERROR: JWT secret file not found - ${JWT_SECRET}"
  exit 1
fi

# Function to start a node (dynamically generate configuration via parameters to avoid code duplication)
start_node() {
  local node_id=$1                  # Node ID (0,1,2)
  local http_port=$2                # HTTP port
  local authrpc_port=$3             # AuthRPC port
  local metrics_port=$4             # Metrics port
  local p2p_port=$5                 # P2P communication port

  local datadir="${BASE_DIR}/node${node_id}"
  local log_file="${LOG_DIR}/node${node_id}.log"

  echo "Starting node ${node_id} ..."
  echo "  Data directory: ${datadir}"
  echo "  Log file: ${log_file}"
  echo "  P2P Port: ${p2p_port} | HTTP Port: ${http_port} | AuthRPC Port: ${authrpc_port}"

  # Start node (run in background with nohup, redirect output to log)
  nohup metis node \
    ${VERBOSITY} \
    -d \
    --datadir="${datadir}" \
    --chain="${GENESIS_PATH}" \
    --http \
    --http.port="${http_port}" \
    --http.addr=0.0.0.0 \
    --http.corsdomain=* \
    --http.api=all \
    --authrpc.addr=0.0.0.0 \
    --authrpc.port="${authrpc_port}" \
    --authrpc.jwtsecret="${JWT_SECRET}" \
    --metrics=0.0.0.0:"${metrics_port}" \
    --discovery.port="${p2p_port}" \
    --port="${p2p_port}" \
    >> "${log_file}" 2>&1 &

  # Record process ID (for later stopping the node)
  echo $! > "${BASE_DIR}/node${node_id}.pid"
}

# Start all nodes (manage port mappings via array for clear configuration)
declare -a NODE_CONFIGS=(
  # NodeID:HTTP_Port:AuthRPC_Port:Metrics_Port:P2P_Port
  "0:8545:8551:9001:30303"
  "1:18545:18551:9101:40303"
  "2:28545:28551:9201:50303"
)

# Iterate through configurations to start nodes
for config in "${NODE_CONFIGS[@]}"; do
  # Parse configuration items (split by colon)
  IFS=':' read -r node_id http_port authrpc_port metrics_port p2p_port <<< "${config}"
  start_node "${node_id}" "${http_port}" "${authrpc_port}" "${metrics_port}" "${p2p_port}"
done

echo "All node startup commands have been issued. Check logs: tail -f ${LOG_DIR}/node*.log"
echo "To stop nodes: kill \$(cat node*.pid) && rm node*.pid"
