#!/usr/bin/env bash

set -e

overhead_file="./extrinsic_overhead_weights.rs"

cargo build --profile release --features runtime-benchmarks --features all-frequency-features --workspace || exit_err

RUNTIME=./target/release/frequency
BENCHMARK="$RUNTIME benchmark overhead "

echo "Creating overhead benchmarks"

$BENCHMARK \
 --chain="frequency-bench" \
 --execution wasm \
 --wasm-execution compiled \
 --weight-path=runtime/common/src/weights

#cargo run --release -p node-bench -- ::node::import::wasm::sr25519::noop::rocksdb::custom --transactions 10000
# cargo run --profile=release -- benchmark overhead --chain="frequency-bench" --execution=wasm --wasm-execution=compiled --weight-path=runtime/common/src/weights/
#cargo run --profile release --features runtime-benchmarks --features all-frequency-features -- benchmark overhead --chain="frequency-bench" --execution=wasm --wasm-execution=compiled --weight-path=runtime/common/src/weights/
# cargo run --profile release --features runtime-benchmarks --features all-frequency-features -- benchmark overhead --chain="frequency-bench" --weight-path=runtime/common/src/weights/
# ${RUNTIME} benchmark overhead --chain="frequency-bench" --execution=wasm --wasm-execution=compiled --weight-path=runtime/common/src/weights/
