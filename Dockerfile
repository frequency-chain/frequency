# Based from https://github.com/paritytech/substrate/blob/master/.maintain/Dockerfile

FROM phusion/baseimage:focal-1.0.0 as builder
LABEL maintainer="MRC Team"
LABEL description="Build stage to create MRC binary."

ARG RUST_TOOLCHAIN=nightly
ENV DEBIAN_FRONTEND=noninteractive
ENV RUST_TOOLCHAIN=$RUST_TOOLCHAIN

ARG PROFILE=release
WORKDIR /mrc

COPY . /mrc

RUN apt-get update && \
	apt-get dist-upgrade -y -o Dpkg::Options::="--force-confold" && \
	apt-get install -y cmake pkg-config libssl-dev git clang libclang-dev

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y && \
	export PATH="$PATH:$HOME/.cargo/bin" && \
	rustup default $RUST_TOOLCHAIN && \
	rustup target add wasm32-unknown-unknown --toolchain $RUST_TOOLCHAIN && \
	cargo build "--$PROFILE"

# ===== SECOND STAGE ======

FROM phusion/baseimage:focal-1.0.0
LABEL maintainer="MRC Team"
LABEL description="Create an image with MRC binary built in first stage."
ARG PROFILE=release

RUN mv /usr/share/ca* /tmp && \
	rm -rf /usr/share/*  && \
	mv /tmp/ca-certificates /usr/share/ && \
	mkdir -p /root/.local/share/mrc && \
    ln -s /root/.local/share/mrc /data

COPY --from=builder /mrc/target/$PROFILE/mrc /usr/local/bin

# checks
RUN ldd /usr/local/bin/mrc && \
	/usr/local/bin/mrc --version

# Shrinking
RUN rm -rf /usr/lib/python* && \
	rm -rf /usr/bin /usr/sbin /usr/share/man

# Add chain resources to image
COPY res /resources/

# USER mrc # see above
EXPOSE 30333 9933 9944
VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/mrc"]
