#!/bin/sh
# set -ex

polkadot_version=$1
rust_toolchain=$2
changelog=$3
runtime_mainnet_info=$4
runtime_paseo_info=$5
is_full_release=$6
metadata_change_summary_file=$6

# Extract the contents of the Summary section from the metadata change summary file, but remove trailing whitespace/blank lines
metadata_change_summary=`sed -n '/SUMMARY/,/^------/p' "$metadata_change_summary_file" | sed '1d;$d' | sed -e :a -e '/^[[:space:]]*$/{$d;N;ba' -e '}'`

CHANGELOG="$changelog" \
    RUST_TOOLCHAIN="$rust_toolchain" \
    POLKADOT_VERSION="$polkadot_version" \
    RUNTIME_MAINNET_INFO="$runtime_mainnet_info" \
    RUNTIME_PASEO_INFO="$runtime_paseo_info" \
    IS_FULL_RELEASE="$is_full_release" \
    METADATA_CHANGE_SUMMARY="$metadata_change_summary" \
    tera -a --env --env-key env --env-only --template release-notes.md.tera
