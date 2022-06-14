#!/usr/bin/env bash

# this script runs the mrc after fetching
# appropriate bootnode IDs

set -e -o pipefail

ctpc="${MRC_BINARY_PATH:-./target/release/mrc-collator}"

if [ ! -x "$ctpc" ]; then
    echo "FATAL: $ctpc does not exist or is not executable"
    exit 1
fi

# name the variable with the incoming args so it isn't overwritten later by function calls
args=( "$@" )

alice="${HOST_NAME:-127.0.0.1}"
bob="${HOST_NAME:-127.0.0.1}"
alice_p2p_port="${ALICE_WS_PORT:-30333}"
alice_rpc_port="${ALICE_RPC_PORT:-9933}"
bob_p2p_port="${BOB_WS_PORT:-30344}"
bob_rpc_port="${BOB_RPC_PORT:-9934}"
chain="${RELAY_CHAIN_SPEC:-./res/rococo-local.json}"


get_id () {
    node="$1"
    port="$2"
    curl -sS \
        -H 'Content-Type: application/json' \
        --data '{"id":1,"jsonrpc":"2.0","method":"system_localPeerId"}' \
        "$node:$port" |\
    jq -r '.result'

}

bootnode () {
    node="$1"
    p2p_port="$2"
    rpc_port="$3"
    id=$(get_id "$node" "$rpc_port")
    if [ -z "$id" ]; then
        echo >&2 "failed to get id for $node"
        exit 1
    fi
    echo "/ip4/$node/tcp/$p2p_port/p2p/$id"
}

args+=( "--" "--wasm-execution=compiled" "--execution=wasm" "--chain=${chain}" "--bootnodes=$(bootnode "$alice" "$alice_p2p_port" "$alice_rpc_port")" "--bootnodes=$(bootnode "$bob" "$bob_p2p_port" "$bob_rpc_port")" )

set -x
"$ctpc" "${args[@]}"
