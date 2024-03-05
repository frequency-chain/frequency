#!/bin/sh
# set -ex

polkadot_version=$1
changelog=$2
runtime_mainnet_info=$3
runtime_paseo_info=$4
runtime_rococo_info=$5

CHANGELOG="$changelog" \
    POLKADOT_VERSION="$polkadot_version" \
    RUNTIME_MAINNET_INFO="$runtime_mainnet_info" \
    RUNTIME_ROCOCO_INFO="$runtime_rococo_info" \
    RUNTIME_PASEO_INFO="$runtime_paseo_info" \
    tera -a --env --env-key env --env-only --template release-notes.md.tera
