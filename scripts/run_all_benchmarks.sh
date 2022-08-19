#!/usr/bin/env bash

if [ "$1"="-h" -o "$1"="--h" -o "$1"="--help" ] ; then
    echo "Usage:  $0 <project-home-dir>"
fi

export RUST_LOG=info
PROJECT=${1:-$HOME/frequency}
CHAIN=$PROJECT/res/genesis/testnet/frequency-spec-rococo-testnet.json
RUNTIME=$PROJECT/target/release/frequency
BENCHMARK="$RUNTIME benchmark pallet "
OUTPUT_DIR=$PROJECT/runtime/frequency/src/weights

function run_benchmark() {
  echo "running benchmarks for $1"
  echo " "
  $BENCHMARK \
  --pallet $1 \
  --extrinsic "*" \
  --chain="$CHAIN" \
  --execution wasm \
  --wasm-execution compiled \
  --steps 50 \
  --repeat 10 \
  --output $OUTPUT_DIR/$1_weights.rs \
  --template=$2
}

cargo build --release  --features runtime-benchmarks --workspace || exit 1
make specs-testnet || exit 1

for external_pallet in orml_vesting pallet_scheduler ; do
  run_benchmark $external_pallet $PROJECT/.maintain/runtime-weight-template.hbs || exit 1
done

for frequency_pallet in pallet_messages pallet_msa pallet_schemas ; do
  run_benchmark $frequency_pallet $PROJECT/.maintain/frame-weight-template.hbs || exit 1
done
