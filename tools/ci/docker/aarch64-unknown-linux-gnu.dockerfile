FROM ghcr.io/cross-rs/aarch64-unknown-linux-gnu:0.2.5

RUN apt-get update && \
	apt-get install -y curl protobuf-compiler && \
	rm -rf /var/lib/apt/lists/*

