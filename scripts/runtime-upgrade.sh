#!/bin/bash

set -x

if [[ -z "$1" || -z $2 ]]; then
  echo "usage: $0 'collator sudo secret' \$ws_provider "
  # fx: $0 '//Alice' ws://0.0.0.0:9944 3 ./res/genesis/local/frequency-rococo-genesis-wasm
  exit 1
fi

echo "🏭 installing subwasm..."
cargo install --locked --git https://github.com/chevdor/subwasm --tag v0.16.1
root_dir=$(git rev-parse --show-toplevel)

cargo build \
  --locked \
  --profile release \
  --package frequency-runtime \
  --target-dir $root_dir/target/ \
  -Z unstable-options


sudo_secret=$1
ws_provider=$2
when=$3
wasm_location=$root_dir/target/release/wbuild/frequency-runtime/frequency_runtime.compact.compressed.wasm

hash=$(subwasm info --json $wasm_location | jq -r .blake2_256)

cd scripts/js/onboard 

yarn && yarn upgrade-auth $ws_provider $sudo_secret $hash 



