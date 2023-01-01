#!/usr/bin/env bash

echo "hello"
ldd --version

## This doesn't work yet, but the idea is to start frequency, move it to the background, go to the schemas directory
## npm install and then deploy schemas, then switch back to the frequency container
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
