#!/usr/bin/env bash

set -e

parachain_id=$1
chain=$2
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

spec_file="./res/genesis/local/${chain}-${parachain_id}.json"
uncompressed_wasm="./res/genesis/local/${chain}-${parachain_id}.wasm"
compressed_wasm="./res/genesis/local/${chain}-${parachain_id}.compressed.wasm"
genesis_state="./res/genesis/local/${chain}-${parachain_id}-genesis-state"
hex_wasm_file="./res/genesis/local/${chain}-${parachain_id}-wasm-hex"

mkdir -p ./res/genesis/local
echo "Building Spec for ${chain} paraid=${parachain_id}"
./target/$profile/frequency build-spec --disable-default-bootnode --chain="${chain}" > "${spec_file}"

cp ./target/$profile/wbuild/frequency-runtime/frequency_runtime.wasm "${uncompressed_wasm}"

subwasm compress "${uncompressed_wasm}" "${compressed_wasm}"

# Update the spec with the compressed WASM
hex_compressed_wasm=`xxd -ps -c 0 "${compressed_wasm}"`
echo -n "0x${hex_compressed_wasm}" > "${hex_wasm_file}"

jq --rawfile code "${hex_wasm_file}" '.genesis.runtimeGenesis.code = $code' "${spec_file}" > "${spec_file}.tmp" && mv "${spec_file}.tmp" "${spec_file}"

echo "Exporting state and wasm for ${chain} paraid=${parachain_id}"
./target/$profile/frequency export-genesis-state --chain="${spec_file}" > "${genesis_state}"

echo "Spec File: ${spec_file}"
echo "Uncompressed wasm: ${uncompressed_wasm}"
echo "Compressed wasm: ${compressed_wasm}"
echo "Genesis State: ${genesis_state}"
echo "Compressed wasm Hex: ${hex_wasm_file}"
