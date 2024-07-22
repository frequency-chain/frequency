#!/bin/sh
# set -ex

polkadot_version=$1
changelog=$2
runtime_mainnet_info=$3
runtime_paseo_info=$4
is_full_release=$5

CHANGELOG="$changelog" \
    POLKADOT_VERSION="$polkadot_version" \
    RUNTIME_MAINNET_INFO="$runtime_mainnet_info" \
    RUNTIME_PASEO_INFO="$runtime_paseo_info" \
    IS_FULL_RELEASE="$is_full_release" \
    tera -a --env --env-key env --env-only --template release-notes.md.tera
