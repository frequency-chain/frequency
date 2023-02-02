#!/bin/sh
# set -ex

polkadot_version=$1
changelog=$2
runtime_rococo_info=$3
runtime_mainnet_info=$4

CHANGELOG="$changelog" \
    POLKADOT_VERSION="$polkadot_version" \
    RUNTIME_ROCOCO_INFO="$runtime_rococo_info" \
    RUNTIME_MAINNET_INFO="$runtime_mainnet_info" \
    tera -a --env --env-key env --env-only --template release-notes.md.tera
