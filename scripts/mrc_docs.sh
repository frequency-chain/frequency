#! /bin/sh

set -e
set -x

./scripts/init.sh install-toolchain
cargo install cargo-deadlinks
cargo doc --no-deps
