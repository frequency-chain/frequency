#!/usr/bin/env bash

export RUST_LOG=info
THIS_DIR=$( dirname -- "$0"; )
PROJECT=${1:-$THIS_DIR/..}
RUNTIME=$PROJECT/target/release/frequency
BENCHMARK="$RUNTIME benchmark pallet "
OUTPUT_DIR=$PROJECT/runtime/frequency/src/weights
SPECS=specs-rococo-2000
CHAIN=$PROJECT/res/genesis/testnet/frequency-spec-rococo-testnet.json

function exit_err() { echo "❌ 💔" ; exit 1; }

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
  --output $OUTPUT_DIR/$2_weights.rs \
  --template=$3
}

cargo build --release --features runtime-benchmarks --workspace || exit_err
make $SPECS || exit_err

for external_pallet in orml_vesting pallet_scheduler pallet_democracy pallet_preimage; do
  run_benchmark $external_pallet $external_pallet $PROJECT/.maintain/runtime-weight-template.hbs || exit_err
done

#for pallet_name in messages msa schemas utility; do
#  echo "run_benchmark pallet_$pallet_name $pallet_name $PROJECT/.maintain/frame-weight-template.hbs" || exit_err
#done
