#!/usr/bin/env bash

set -e

docker run parity/polkadot:v1.0.0 build-spec --disable-default-bootnode --chain rococo-local --raw > ./resources/rococo-local.json

