#!/bin/sh
# set -ex

polkadot_version=$1
changelog=$2

CHANGELOG="$changelog" POLKADOT_VERSION="$polkadot_version" tera -a --env --env-key env --env-only --template release-notes.md.tera
