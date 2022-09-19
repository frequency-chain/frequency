#!/usr/bin/env bash

set -e

echo $(pwd)

docker run parity/polkadot:v0.9.27 build-spec --disable-default-bootnode --chain rococo-local --raw > ./resources/rococo-local.json
