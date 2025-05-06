# NOTE: If you make changes in this file, be sure to update IMAGE_VERSION in merge-pr.yml
# ci-base-image is published IF and ONLY IF changes are detected in this dockerfile.

FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive
LABEL maintainer="Frequency"
LABEL description="Frequency CI Base Image"
# Image version is set by the CI pipeline in merge-pr.yml
ARG IMAGE_VERSION
LABEL version="${IMAGE_VERSION}"
LABEL org.opencontainers.image.description="Frequency CI Base Image"

WORKDIR /ci
# Install rustup and needed build tools
RUN apt update && \
  apt install --no-install-recommends -y rustup curl build-essential libclang-dev protobuf-compiler git file jq clang cmake ca-certificates && \
  update-ca-certificates && \
  apt remove -y --auto-remove && \
  rm -rf /var/lib/apt/lists/*

ARG RUST_VERSION
LABEL rust.version="${RUST_VERSION}"

# Install architecture-specific targets
# rustup set auto-self-update disable is required as we are installing rustup via apt
# The list of components should match the rust-toolchain.toml file
RUN rustup --version && \
  echo $RUSTUP_HOME && \
  rustup set auto-self-update disable && \
  rustup toolchain install "${RUST_VERSION}" && \
  rustup target add x86_64-unknown-linux-gnu --toolchain "${RUST_VERSION}" && \
  rustup target add aarch64-unknown-linux-gnu --toolchain "${RUST_VERSION}" && \
  rustup target add wasm32v1-none --toolchain "${RUST_VERSION}" && \
  rustup component add clippy rust-docs rustfmt rustc-dev rustc rust-src --toolchain "${RUST_VERSION}"

RUN git config --system --add safe.directory /__w/frequency/frequency
