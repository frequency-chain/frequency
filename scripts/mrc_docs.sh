#! /bin/sh

set -e
set -x

./scripts/init.sh install-toolchain
RUSTDOCFLAGS="--enable-index-page -Zunstable-options" cargo doc --no-deps
