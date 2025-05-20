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
  apt install --no-install-recommends -y gpg sudo curl build-essential libclang-dev protobuf-compiler git file jq clang cmake ca-certificates && \
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
