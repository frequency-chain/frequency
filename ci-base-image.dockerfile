# NOTE: If you make changes in this file, be sure to update IMAGE_VERSION in merge-pr.yml
# ci-base-image is published IF and ONLY IF changes are detected in this dockerfile.

FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive
LABEL maintainer="Frequency"
LABEL description="Frequency CI Base Image"
# Image version is set by the CI pipeline in merge-pr.yml
ARG IMAGE_VERSION
LABEL version="${IMAGE_VERSION}"
LABEL org.opencontainers.image.description "Frequency CI Base Image"
ARG RUST_VERSION
LABEL rust.version="${RUST_VERSION}"

WORKDIR /ci
RUN apt-get update && \
	apt-get install -y curl protobuf-compiler build-essential libclang-dev git file jq clang cmake && \
	curl -fsSL https://get.docker.com -o get-docker.sh && sh get-docker.sh && \
	rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/home/runner/.cargo/bin:/root/.cargo/bin:${PATH}"
ENV RUSTUP_HOME="/root/.cargo"
ENV CARGO_HOME="/root/.cargo"
RUN rustup toolchain install $NIGHTLY_VERSION
RUN rustup target add x86_64-unknown-linux-gnu --toolchain $NIGHTLY_VERSION
RUN rustup target add wasm32-unknown-unknown --toolchain $NIGHTLY_VERSION
RUN rustup component add rust-src --toolchain $NIGHTLY_VERSION

RUN git config --system --add safe.directory /__w/frequency/frequency
