#!/bin/sh
# set -ex

changelog=$1

CHANGELOG="$changelog" POLKADOT_VERSION="$(make version-polkadot)" tera -a --env --env-key env --env-only --template release-notes.md.tera
