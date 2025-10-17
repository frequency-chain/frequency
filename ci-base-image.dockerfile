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

# Some Ubuntu images have an ubuntu user - don't error if it doesn't exist
RUN userdel -r ubuntu || true

# Create a non-root user and give permissions
RUN useradd -u 1001 -d /home/runner -m -s /bin/bash runner

# Install rustup and needed build tools
RUN apt update && \
  apt install --no-install-recommends -y gpg gpg-agent sudo curl build-essential libclang-dev protobuf-compiler git file jq clang cmake ca-certificates && \
  update-ca-certificates && \
  apt remove -y --auto-remove && \
  rm -rf /var/lib/apt/lists/* && \
  echo "runner ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers

RUN git config --system --add safe.directory /__w/frequency/frequency

# Switch to non-root by default
USER runner
WORKDIR /home/runner

ARG TARGETARCH
ARG RUST_VERSION
LABEL rust.version="${RUST_VERSION}"

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y --profile minimal --default-toolchain ${RUST_VERSION}
ENV PATH="/home/runner/.cargo/bin:${PATH}"
ENV RUSTUP_HOME="/home/runner/.cargo"
ENV CARGO_HOME="/home/runner/.cargo"

# Install architecture-specific targets
# rustup set auto-self-update disable is required as we are installing rustup via apt
# The list of components should match the rust-toolchain.toml file
RUN case "${TARGETARCH}" in \
  amd64) RUST_ARCH="x86_64" ;; \
  arm64) RUST_ARCH="aarch64" ;; \
  *) echo "Unsupported architecture: ${TARGETARCH}" && exit 1 ;; \
  esac && \
  echo "Installing toolchain for arch: ${RUST_ARCH}" && \
  rustup --version && \
  rustup set auto-self-update disable && \
  rustup toolchain install "${RUST_VERSION}-${RUST_ARCH}-unknown-linux-gnu" && \
  rustup +"${RUST_VERSION}-${RUST_ARCH}-unknown-linux-gnu" target add x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu wasm32v1-none && \
  rustup +"${RUST_VERSION}-${RUST_ARCH}-unknown-linux-gnu" component add clippy rust-docs rustfmt rustc-dev rustc rust-src && \
  rustup +${RUST_VERSION} show

# Install compilers required by Polkadot SDK stable2509+ for pallet-revive
# Both solc (Ethereum Solidity) and resolc (Revive/PolkaVM) are needed
# as pallet-revive-fixtures compiles contracts for both EVM and PVM
RUN mkdir -p /home/runner/.local/bin && \
  case "${TARGETARCH}" in \
    amd64) \
      curl -L https://github.com/ethereum/solidity/releases/download/v0.8.28/solc-static-linux -o /home/runner/.local/bin/solc && \
      curl -L https://github.com/paritytech/revive/releases/download/v0.4.1/resolc-x86_64-unknown-linux-musl -o /home/runner/.local/bin/resolc ;; \
    arm64) \
      curl -L https://github.com/ethereum/solidity/releases/download/v0.8.28/solc-static-linux -o /home/runner/.local/bin/solc && \
      curl -L https://github.com/paritytech/revive/releases/download/v0.4.1/resolc-aarch64-unknown-linux-musl -o /home/runner/.local/bin/resolc ;; \
    *) echo "Unsupported architecture for solc/resolc: ${TARGETARCH}" && exit 1 ;; \
  esac && \
  chmod +x /home/runner/.local/bin/solc /home/runner/.local/bin/resolc

ENV PATH="/home/runner/.local/bin:${PATH}"
