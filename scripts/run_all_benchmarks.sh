#!/usr/bin/env bash

export RUST_LOG=info
THIS_DIR=$( dirname -- "$0"; )
PROJECT=${1:-$THIS_DIR/..}
RUNTIME=$PROJECT/target/production/frequency
BENCHMARK="$RUNTIME benchmark pallet "
SPECS=specs-rococo-4044
EXTERNAL_PALLETS=(orml_vesting pallet_scheduler pallet_democracy pallet_preimage pallet_utility)
CUSTOM_PALLETS=(messages msa schemas)

function exit_err() { echo "❌ 💔" ; exit 1; }

function run_benchmark() {
  echo "running benchmarks for $1"
  echo " "
  $BENCHMARK \
  --pallet $1 \
  --extrinsic "*" \
  --chain="frequency" \
  --execution wasm \
  --wasm-execution compiled \
  --steps 50 \
  --repeat 10 \
  --output=$2 \
  --template=$3
}

cargo build --profile production --features runtime-benchmarks --features all-frequency-features --workspace || exit_err

for external_pallet in "${EXTERNAL_PALLETS[@]}"; do
  output=${PROJECT}/runtime/common/src/weights/${external_pallet}.rs
  template=${PROJECT}/.maintain/runtime-weight-template.hbs
  run_benchmark ${external_pallet} ${output} ${template} || exit_err
done

for pallet_name in "${CUSTOM_PALLETS[@]}"; do
  output=${PROJECT}/pallets/${pallet_name}/src/weights.rs
  template=${PROJECT}/.maintain/frame-weight-template.hbs
  run_benchmark pallet_${pallet_name} ${output} ${template} || exit_err
done
