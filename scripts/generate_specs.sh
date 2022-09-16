#!/usr/bin/env bash

set -e

parachain_id=$1
build_step=$2
profile=$3
if [[ $parachain_id == "" ]]; then
  echo "Chain Name or Parachain ID argument not provided"
  exit 1
fi

BUILT_TARGET=./target/$profile/frequency
if [ ! -x "$BUILT_TARGET" ]; then
    echo "FATAL: $BUILT_TARGET does not exist, or not executable, rebuild binary to continue"
    exit 1
fi
case $build_step in
  rococo-2000)
    mkdir -p ./res/genesis/local
    echo "Building Spec for  frequency rococo localnet paraid=2000"
    $PWD/target/$profile/frequency build-spec --disable-default-bootnode --chain=frequency-local > ./res/genesis/local/frequency-spec-rococo.json
    sed -i.bu "s/\"parachainId\": 2000/\"parachainId\": $parachain_id/g" ./res/genesis/local/frequency-spec-rococo.json
    sed -i.bu "s/\"para_id\": 2000/\"para_id\": $parachain_id/g" ./res/genesis/local/frequency-spec-rococo.json
    $PWD/target/$profile/frequency build-spec --raw --disable-default-bootnode --chain ./res/genesis/local/frequency-spec-rococo.json > ./res/genesis/local/rococo-local-frequency-2000-raw.json
    rm ./res/genesis/local/frequency-spec-rococo.json.bu

    echo "Exporting state and wasm for frequency rococo localnet paraid=2000"
    $PWD/target/$profile/frequency export-genesis-state --chain ./res/genesis/local/rococo-local-frequency-2000-raw.json > ./res/genesis/local/frequency-rococo-genesis-state
    $PWD/target/$profile/frequency export-genesis-wasm --chain ./res/genesis/local/rococo-local-frequency-2000-raw.json > ./res/genesis/local/frequency-rococo-genesis-wasm
    ;;
  rococo-4044)
    mkdir -p ./res/genesis/testnet
    echo "Building Spec for for frequency rococo testnet paraid=4044"
    $PWD/target/$profile/frequency build-spec --chain=frequency-rococo --disable-default-bootnode > ./res/genesis/testnet/frequency-spec-rococo-testnet.json
    sed -i.bu "s/\"parachainId\": 4044/\"parachainId\": $parachain_id/g" ./res/genesis/testnet/frequency-spec-rococo-testnet.json
    sed -i.bu "s/\"para_id\": 4044/\"para_id\": $parachain_id/g" ./res/genesis/testnet/frequency-spec-rococo-testnet.json
    $PWD/target/$profile/frequency build-spec --raw --disable-default-bootnode --chain ./res/genesis/testnet/frequency-spec-rococo-testnet.json > ./res/genesis/testnet/rococo-testnet-frequency-raw.json
    rm ./res/genesis/testnet/frequency-spec-rococo-testnet.json.bu

    echo "Exporting state and wasm for frequency rococo testnet paraid=4044"
    $PWD/target/$profile/frequency export-genesis-state --chain ./res/genesis/testnet/rococo-testnet-frequency-raw.json > ./res/genesis/testnet/frequency-rococo-testnet-genesis-state
    $PWD/target/$profile/frequency export-genesis-wasm --chain ./res/genesis/testnet/rococo-testnet-frequency-raw.json > ./res/genesis/testnet/frequency-rococo-testnet-genesis-wasm
    ;;

  mainnet)
    mkdir -p ./res/genesis/mainnet
    echo "Building Spec for frequency mainnet"
    $PWD/target/$profile/frequency build-spec --chain=frequency --disable-default-bootnode > ./res/genesis/mainnet/frequency.json
    sed -i.bu "s/\"parachainId\": 999/\"parachainId\": $parachain_id/g" ./res/genesis/mainnet/frequency.json
    sed -i.bu "s/\"para_id\": 999/\"para_id\": $parachain_id/g" ./res/genesis/mainnet/frequency.json
    $PWD/target/$profile/frequency build-spec --raw --disable-default-bootnode --chain ./res/genesis/mainnet/frequency.json > ./res/genesis/mainnet/frequency-raw.json
    rm ./res/genesis/mainnet/frequency.json.bu

    echo "Exporting state and wasm for frequency mainnet"
    $PWD/target/$profile/frequency export-genesis-state --chain ./res/genesis/mainnet/frequency-raw.json > ./res/genesis/mainnet/frequency-state
    $PWD/target/$profile/frequency export-genesis-wasm --chain ./res/genesis/mainnet/frequency-raw.json > ./res/genesis/mainnet/frequency-wasm
    ;;
  *)
    echo "Unknown build step: $build_step"
    exit 1
    ;;

esac
