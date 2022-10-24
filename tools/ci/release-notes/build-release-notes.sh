#!/bin/sh
set -ex
# echo "arg 1: $1"
# echo "arg 2: $2"

# args=("$@")
# echo "args: ${args[@]}"
# echo "@: $@"
# echo $@
# echo "args[@]: ${args[@]}"

# frequency_ref2 = $1
# frequency_ref1 = $2 || 'UNKNOWN'

tera --env --env-key env --template release-notes.md.tera context.json
