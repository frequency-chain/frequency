#! /bin/sh

set -e
set -x

./scripts/init.sh install-toolchain
cargo doc --no-deps
