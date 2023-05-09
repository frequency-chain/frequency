#!/usr/bin/env bash

set -eux

PACKAGE="${PACKAGE:-frequency-runtime}" # Need to replicate job for all runtimes
RUNTIME_DIR="${RUNTIME_DIR:-runtime/frequency-rococo}"
SRT_TOOL_VERSION="${SRT_TOOL_VERSION:-1.62.0}"

# Enable warnings about unused extern crates
export RUSTFLAGS=" -W unused-extern-crates"

rustc --version
rustup --version
cargo --version

case $TARGET in
  build-node)
    cargo build --release --features frequency-no-relay "$@"
    ;;

  build-runtime)
    export RUSTC_VERSION=$SRT_TOOL_VERSION
    echo "Building runtime with rustc version $RUSTC_VERSION"
    docker run --rm -e PACKAGE=$PACKAGE -e RUNTIME_DIR=$RUNTIME_DIR -e PROFILE=$PROFILE -e BUILD_OPTS="--features on-chain-release-build,no-metadata-docs" -v $PWD:/build -v /tmp/cargo:/cargo-home paritytech/srtool:$RUSTC_VERSION build
    ;;

  tests)
    cargo test --features runtime-benchmarks,frequency-lint-check,std --workspace --release
    ;;

  lint)
    cargo fmt -- --check
    SKIP_WASM_BUILD=1 env -u RUSTFLAGS cargo clippy --features frequency-lint-check,std -- -D warnings
    RUSTDOCFLAGS="--enable-index-page --check -Zunstable-options" cargo doc --no-deps
    ;;
esac
