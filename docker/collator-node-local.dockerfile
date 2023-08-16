# Docker image for running collator node node locally against the local relay chain.
# Requires to run from repository root and to copy the binary in the build folder.
FROM --platform=linux/amd64 ubuntu:20.04
LABEL maintainer="Frequency"
LABEL description="Frequency collator node for local relay chain"

WORKDIR /frequency

RUN apt-get update && \
	apt-get install -y jq apt-utils apt-transport-https \
		software-properties-common readline-common curl vim wget gnupg gnupg2 \
		gnupg-agent ca-certificates tini && \
	rm -rf /var/lib/apt/lists/*

# For local testing only
# COPY target/release/frequency.amd64 ./target/release/frequency
COPY target/release/frequency ./target/release/
RUN chmod +x target/release/frequency

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

# 9944 rpc port
# 30333 p2p port
# 9615 for Telemetry (prometheus)
EXPOSE 9944 30333 9615

ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/bin/bash", "./scripts/init.sh", "start-frequency-container"]


