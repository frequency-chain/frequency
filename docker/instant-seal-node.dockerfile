# Docker image for running Frequency parachain node container (with collating)
# locally in instant seal mode. Requires to run from repository root and to copy
# the binary in the build folder.
# This is the build stage for Polkadot. Here we create the binary in a temporary image.
FROM --platform=linux/amd64 ubuntu:22.04 AS base

LABEL maintainer="Frequency"
LABEL description="Frequency collator node in instant seal mode"

RUN apt-get update && apt-get install -y ca-certificates jq curl && update-ca-certificates

# This is the 2nd stage: a very small image where we copy the Frequency binary
FROM --platform=linux/amd64 ubuntu:22.04

# We want jq and curl in the final image, but we don't need the support files
RUN apt-get update && \
	apt-get install -y jq curl && \
	apt-get clean && \
	rm -rf /usr/share/doc /usr/share/man /usr/share/zsh

RUN useradd -m -u 1000 -U -s /bin/sh -d /frequency frequency && \
	mkdir -p /data /frequency/.local/share && \
	chown -R frequency:frequency /data && \
	ln -s /data /frequency/.local/share/frequency

USER frequency

COPY --from=base /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
# For local testing only
# COPY --chown=frequency target/release/frequency.amd64 ./frequency/frequency
COPY --chown=frequency target/release/frequency ./frequency/
RUN chmod +x ./frequency/frequency

# 9944 for RPC call
# 30333 for p2p
# 9615 for Telemetry (prometheus)
EXPOSE 9944 30333 9615

VOLUME ["/data"]

ENTRYPOINT ["/frequency/frequency", \
	# Required params for starting the chain
	"--dev", \
	"-lruntime=debug", \
	"--no-telemetry", \
	"--no-prometheus", \
	"--port=30333", \
	"--rpc-port=9944", \
	"--rpc-external", \
	"--rpc-cors=all", \
	"--rpc-methods=Unsafe", \
	"--base-path=/data" \
	]

# Params which can be overriden from CLI
CMD ["--sealing=instant"]
