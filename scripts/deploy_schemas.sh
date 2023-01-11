#!/bin/bash

set -m
echo "hello"
ldd --version

chmod +x /frequency/frequency
/frequency/frequency --dev \
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
    --tmp \
    &

cd /frequency/schemas
npm install && npm run deploy

fg %1
