#!/usr/bin/env bash

# this script runs the frequency after fetching
# appropriate bootnode IDs

set -e -o pipefail

ctpc="${Frequency_BINARY_PATH:-./target/debug/frequency}"

if [ ! -x "$ctpc" ]; then
    echo "FATAL: $ctpc does not exist or is not executable"
    exit 1
fi

# name the variable with the incoming args so it isn't overwritten later by function calls
args=( "$@" )

alice="${HOST_ALICE:-127.0.0.1}"
bob="${HOST_BOB:-127.0.0.1}"
alice_rpc_port="${ALICE_RPC_PORT:-9946}"
bob_rpc_port="${BOB_RPC_PORT:-9947}"
chain="${RELAY_CHAIN_SPEC:-./resources/rococo-local.json}"

get_bootnode () {
    node="$1"
    port="$2"
    SELECT_INDEX=`[[ "$alice" == "127.0.0.1" ]] && echo "0" || echo "1"`
    curl -sS \
        -H 'Content-Type: application/json' \
        --data '{"id":1,"jsonrpc":"2.0","method":"system_localListenAddresses"}' \
        "$node:$port" |\
    tee |\
    jq -r '.result['$SELECT_INDEX'] // ""'

}

bootnode () {
    node="$1"
    rpc_port="$2"
    bootnode=$(get_bootnode "$node" "$rpc_port")
    if [ -z "$bootnode" ]; then
        echo >&2 "failed to get id for $node"
        # curl -vsS \
        # -H 'Content-Type: application/json' \
        # --data '{"id":1,"jsonrpc":"2.0","method":"localListenAddresses"}' \
        # "$node:$rpc_port"
        exit 1
    fi
    >&2 echo "Bootnode: $bootnode"
    echo "$bootnode"
}

args+=( "--" "--wasm-execution=compiled" "--chain=${chain}" "--bootnodes=$(bootnode "$alice" "$alice_rpc_port")" "--bootnodes=$(bootnode "$bob" "$bob_rpc_port")" )

set -x
"$ctpc" "${args[@]}"
