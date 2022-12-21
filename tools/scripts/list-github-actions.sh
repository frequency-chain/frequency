#!/bin/sh

# set -ex

grep -o -r --no-filename --include=\*.yml "uses:.*" ./.github | \
    cut -d ":" -f 2 | tr -d " " | sort | uniq
