# Docker image for running collator node node locally against the local relay chain.
# Multi-architecture support for amd64 and arm64
FROM ubuntu:24.04
LABEL maintainer="Frequency"
LABEL description="Frequency collator node for local relay chain"

# Some Ubuntu images have an ubuntu user
RUN userdel -r ubuntu || true

WORKDIR /frequency

RUN apt-get update && \
	apt-get install -y jq apt-utils apt-transport-https \
	software-properties-common readline-common curl vim wget gnupg gnupg2 \
	gnupg-agent ca-certificates tini file && \
	rm -rf /var/lib/apt/lists/*

# Create the directory structure expected by the scripts
RUN mkdir -p /frequency/target/release

# Copy the appropriate binary based on the target platform
COPY linux/${TARGETARCH}/frequency /frequency/target/release/frequency
RUN chmod +x /frequency/target/release/frequency

RUN ls ./target/release

# Checks
RUN ls -lah /
RUN file ./target/release/frequency && \
	./target/release/frequency --version

# Add chain resources to image
COPY resources ./resources/
COPY scripts ./scripts/

RUN chmod +x ./scripts/run_collator.sh
RUN chmod +x ./scripts/init.sh
RUN chmod +x ./scripts/healthcheck.sh

ENV Frequency_BINARY_PATH=./target/release/frequency

HEALTHCHECK --interval=300s --timeout=75s --start-period=30s --retries=3 \
	CMD ["./scripts/healthcheck.sh"]

VOLUME ["/data"]

# Frequency Chain Ports
# 9944 for Websocket and Rpc
# 30333 for P2P
# 9615 for Telemetry (prometheus)
# Relay Chain Ports
# 9945 for Websocket and Rpc
# 30334 for P2P
# 9616 for Telemetry (prometheus)
EXPOSE 9944 30333 9615 9945 30334 9616

ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/bin/bash", "./scripts/init.sh", "start-frequency-container"]
