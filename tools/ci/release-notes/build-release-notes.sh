#!/bin/sh
# set -ex
# echo "arg 1: $1"
# echo "arg 2: $2"

# sanitized_change_log=${$1//\`/}
# echo "sanitized change_log:$sanitized_change_log"
changelog=$1

# args=("$@")
# echo "args: ${args[@]}"
# echo "@: $@"
# echo $@
# echo "args[@]: ${args[@]}"

# frequency_ref2 = $1
# frequency_ref1 = $2 || 'UNKNOWN'

# tera --env --env-key env --template release-notes.md.tera context.json
# tera --env --env-key env --env-first --template release-notes.md.tera
CHANGELOG="$changelog" tera --env --env-key env --env-only --template release-notes.md.tera
