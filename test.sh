#!/usr/bin/env bash

echo "hello"
ls
./frequency/frequency --dev \
    -lruntime=debug \
    --instant-sealing \
    --wasm-execution=compiled \
    --execution=wasm \
    --no-telemetry \
    --no-prometheus \
    --port=30333 \
    --rpc-port=9933 \
    --ws-port=9944 \
    --rpc-external \
    --rpc-cors=al \
    --ws-external \
    --rpc-methods=Unsafe \
    --tmp

