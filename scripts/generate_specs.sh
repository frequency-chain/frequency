#!/usr/bin/env bash

set -e

parachain_id=$1
build_step=$2

if [[ $parachain_id == "" ]]; then
  echo "Chain Name or Parachain ID argument not provided"
  exit 1
fi

BUILT_TARGET=./target/release/mrc-collator
if [ ! -x "$BUILT_TARGET" ]; then
    echo "FATAL: $BUILT_TARGET does not exist, or not executable, rebuild binary to continue"
    exit 1
fi

if [[ $build_step == "true" ]]; then
  echo "Building Spec for mrc as rococo collator"
  $PWD/target/release/mrc-collator build-spec --disable-default-bootnode > ./res/genesis/mrc-spec-rococo.json
  sed -i.bu "s/\"parachainId\": 2000/\"parachainId\": $parachain_id/g" ./res/genesis/mrc-spec-rococo.json
  sed -i.bu "s/\"para_id\": 2000/\"para_id\": $parachain_id/g" ./res/genesis/mrc-spec-rococo.json
  $PWD/target/release/mrc-collator build-spec --raw --disable-default-bootnode --chain ./res/genesis/mrc-spec-rococo.json > ./res/genesis/rococo-local-mrc-2000-raw.json
  rm ./res/genesis/mrc-spec-rococo.json.bu
fi

echo "Exporting state and wasm for mrc"
$PWD/target/release/mrc-collator export-genesis-state --chain ./res/genesis/rococo-local-mrc-2000-raw.json > ./res/genesis/mrc-rococo-genesis-state
$PWD/target/release/mrc-collator export-genesis-wasm --chain ./res/genesis/rococo-local-mrc-2000-raw.json > ./res/genesis/mrc-rococo-genesis-wasm
