#!/usr/bin/env bash

export RUST_LOG=info
THIS_DIR=$( dirname -- "$0"; )
PROJECT=${1:-$THIS_DIR/..}
RUNTIME=$PROJECT/target/production/frequency
BENCHMARK="$RUNTIME benchmark pallet "
# TODO: pallet_collator_selection benchmarks fail due to errors in the actual benchmark code. See Issue #608
EXTERNAL_PALLETS=(pallet_collective orml_vesting pallet_balances pallet_timestamp pallet_session pallet_scheduler pallet_democracy pallet_treasury pallet_preimage pallet_utility)
CUSTOM_PALLETS=(messages msa schemas capacity)

function exit_err() { echo "❌ 💔" ; exit 1; }

function run_benchmark() {
  echo "running benchmarks for $1"
  echo " "
  $BENCHMARK \
  --pallet $1 \
  --extrinsic "*" \
  --chain="frequency-bench" \
  --execution wasm \
  --heap-pages=4096 \
  --wasm-execution compiled \
  --steps=$2 \
  --repeat=$3 \
  --output=$4 \
  --template=$5
}

cargo build --profile production --features runtime-benchmarks --features all-frequency-features --workspace || exit_err

for external_pallet in "${EXTERNAL_PALLETS[@]}"; do
  output=${PROJECT}/runtime/common/src/weights/${external_pallet}.rs
  steps=50
  repeat=20
  template=${PROJECT}/.maintain/runtime-weight-template.hbs
  run_benchmark ${external_pallet} ${steps} ${repeat} ${output} ${template} || exit_err
done

for pallet_name in "${CUSTOM_PALLETS[@]}"; do
  steps=20
  repeat=10
  template=${PROJECT}/.maintain/frame-weight-template.hbs
  output=${PROJECT}/pallets/${pallet_name}/src/weights.rs
  run_benchmark pallet_${pallet_name} ${steps} ${repeat} ${output} ${template} || exit_err
done
