#!/usr/bin/env bash

# this script runs the mrc after fetching
# appropriate bootnode IDs

set -e -o pipefail

ctpc="./target/release/mrc-collator"

if [ ! -x "$ctpc" ]; then
    echo "FATAL: $ctpc does not exist or is not executable"
    exit 1
fi

# name the variable with the incoming args so it isn't overwritten later by function calls
args=( "$@" )

set -x
"$ctpc" "${args[@]}"
