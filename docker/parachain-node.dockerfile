# Docker image for running Frequency parachain node container (without collating)
# for Rococo testnet or Mainnet. Requires to run from repository root and to copy
# the binary in the build folder.
# This is the build stage for Polkadot. Here we create the binary in a temporary image.
FROM --platform=linux/amd64 ubuntu:22.04 AS base
LABEL maintainer="Frequency"
LABEL description="Frequency parachain node for Rococo testnet and Mainnet"

RUN apt-get update && apt-get install -y ca-certificates && update-ca-certificates

# This is the 2nd stage: a very small image where we copy the Frequency binary
FROM --platform=linux/amd64 ubuntu:22.04

RUN useradd -m -u 1000 -U -s /bin/sh -d /frequency frequency && \
	mkdir -p /chain-data /frequency/.local/share && \
	chown -R frequency:frequency /chain-data && \
	ln -s /chain-data /frequency/.local/share/frequency

USER frequency

COPY --from=base /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
# For local testing only
# COPY --chown=frequency target/release/frequency.amd64 ./frequency/frequency
COPY --chown=frequency target/release/frequency ./frequency/

# 9944 for Websocket and Rpc
# 30333 for P2P
# 9615 for Telemetry (prometheus)
EXPOSE 9944 30333 9615

VOLUME ["/chain-data"]

ENTRYPOINT ["/frequency/frequency"]

# Params which can be overriden from CLI
# CMD ["", "", ...]


