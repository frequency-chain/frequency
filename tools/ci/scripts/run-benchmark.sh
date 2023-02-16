#!/usr/bin/env bash

set -e

while getopts p:w: flag
do
    case "${flag}" in
        p) pallet=${OPTARG};;
    esac
done

if [ -z "$pallet" ]
  then
    echo "Error: No Pallet Provided" >&2;
    exit 1;
fi

weights_file="./pallets/$pallet/src/weights.rs";
if [ ! -f "$weights_file" ]
  then
    echo "Error: File $weights_file does not exist." >&2;
    exit 1;
fi

# Change to release for faster testing
PROFILE=production

cargo build --profile "$PROFILE" --features runtime-benchmarks --features all-frequency-features --workspace || exit_err

RUNTIME=./target/$PROFILE/frequency
BENCHMARK="$RUNTIME benchmark pallet "

echo "Creating benchmarks for $pallet"

RUST_LOG=runtime=debug $BENCHMARK \
  --pallet pallet_$pallet \
  --extrinsic "*" \
  --chain="frequency-bench" \
  --execution wasm \
  --wasm-execution compiled \
  --steps 50 \
  --repeat 10 \
  --output=$weights_file \
  --template=./.maintain/frame-weight-template.hbs

