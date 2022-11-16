#!/bin/sh
# set -ex

changelog=$1

CHANGELOG="$changelog" tera --env --env-key env --env-only --template release-notes.md.tera
