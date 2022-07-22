#! /bin/sh

set -e
set -x

cargo build --features runtime-benchmarks --release

./target/release/frequency build-spec > ./res/genesis/frequency-weights.json

./target/release/frequency benchmark pallet --chain ./res/genesis/frequency-test.json --execution=wasm --wasm-execution=compiled --pallet pallet_messages --extrinsic '*' --steps 20 --repeat 5 --output ./pallets/messages/src/weights.rs --template=./.maintain/frame-weight-template.hbs
