#!/usr/bin/env bash

set -e

frequency_rpc_port="${Frequency_RPC_PORT:-11936}"

node="127.0.0.1"
port="$frequency_rpc_port"
curl -sS \
    -H 'Content-Type: application/json' \
    --data '{"id":1,"jsonrpc":"2.0","method":"system_health"}' \
    "$node:$port" |\
  jq -e -r '.result'
