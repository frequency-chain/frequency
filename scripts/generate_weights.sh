#! /bin/sh

set -e
set -x

while getopts p:w: flag
do
    case "${flag}" in
        p) pallet=${OPTARG};;
        w) weights_file=./pallets/${OPTARG};;
    esac
done

if [ -z "$pallet" ]
  then 
    echo "Error: No Pallet Provided" >&2;
    exit 1;
fi
if [ -z "$weights_file" ]
  then
    weights_file="./pallets/$pallet/src/weights.rs";
    echo "No weight file provided, using default: $weights_file"
fi
if [ ! -f "$weights_file" ]
  then
    echo "Error: Default file $weights_file does not exist." >&2;
    exit 1;
fi

cargo build --features runtime-benchmarks --release

./target/release/frequency build-spec > ./res/genesis/frequency-weights.json

./target/release/frequency benchmark pallet --chain ./res/genesis/frequency-weights.json --execution=wasm --wasm-execution=compiled --pallet pallet_$pallet --extrinsic '*' --steps 20 --repeat 5 --output $weights_file --template=./.maintain/frame-weight-template.hbs
