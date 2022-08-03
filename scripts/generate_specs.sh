#!/usr/bin/env bash

set -e

parachain_id=$1
build_step=$2

if [[ $parachain_id == "" ]]; then
  echo "Chain Name or Parachain ID argument not provided"
  exit 1
fi

BUILT_TARGET=./target/release/frequency
if [ ! -x "$BUILT_TARGET" ]; then
    echo "FATAL: $BUILT_TARGET does not exist, or not executable, rebuild binary to continue"
    exit 1
fi
case $build_step in
  build-local)
    echo "Building Spec for frequency as rococo local"
    $PWD/target/release/frequency build-spec --disable-default-bootnode > ./res/genesis/local/frequency-spec-rococo.json
    sed -i.bu "s/\"parachainId\": 2000/\"parachainId\": $parachain_id/g" ./res/genesis/local/frequency-spec-rococo.json
    sed -i.bu "s/\"para_id\": 2000/\"para_id\": $parachain_id/g" ./res/genesis/local/frequency-spec-rococo.json
    $PWD/target/release/frequency build-spec --raw --disable-default-bootnode --chain ./res/genesis/local/frequency-spec-rococo.json > ./res/genesis/local/rococo-local-frequency-2000-raw.json
    rm ./res/genesis/local/frequency-spec-rococo.json.bu

    echo "Exporting state and wasm for frequency local"
    $PWD/target/release/frequency export-genesis-state --chain ./res/genesis/local/rococo-local-frequency-2000-raw.json > ./res/genesis/local/frequency-rococo-genesis-state
    $PWD/target/release/frequency export-genesis-wasm --chain ./res/genesis/local/rococo-local-frequency-2000-raw.json > ./res/genesis/local/frequency-rococo-genesis-wasm
    ;;
  build-testnet)
    echo "Building Spec for frequency as rococo testnet"
    $PWD/target/release/frequency build-spec --chain=frequency_rococo --disable-default-bootnode > ./res/genesis/testnet/frequency-spec-rococo-testnet.json
    sed -i.bu "s/\"parachainId\": 4044/\"parachainId\": $parachain_id/g" ./res/genesis/testnet/frequency-spec-rococo-testnet.json
    sed -i.bu "s/\"para_id\": 4044/\"para_id\": $parachain_id/g" ./res/genesis/testnet/frequency-spec-rococo-testnet.json
    $PWD/target/release/frequency build-spec --raw --disable-default-bootnode --chain=frequency_rococo ./res/genesis/testnet/frequency-spec-rococo-testnet.json > ./res/genesis/testnet/rococo-testnet-frequency-raw.json
    rm ./res/genesis/testnet/frequency-spec-rococo.json.bu

    echo "Exporting state and wasm for frequency testnet"
    $PWD/target/release/frequency export-genesis-state --chain=frequency_rococo ./res/genesis/testnet/rococo-testnet-frequency-raw.json > ./res/genesis/testnet/frequency-rococo-testnet-genesis-state
    $PWD/target/release/frequency export-genesis-wasm --chain=frequency_rococo ./res/genesis/testnet/rococo-testnet-frequency-raw.json > ./res/genesis/testnet/frequency-rococo-testnet-genesis-wasm
    ;;
  *)
    echo "Unknown build step: $build_step"
    exit 1
    ;;
esac
