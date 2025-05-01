# Docker image for running Frequency parachain node container (without collating).
# Multi-architecture support for amd64 and arm64
FROM ubuntu:24.04 AS base
LABEL maintainer="Frequency"
LABEL description="Frequency Parachain Node"

RUN apt-get update && apt-get install -y ca-certificates && update-ca-certificates

# This is the 2nd stage: a very small image where we copy the Frequency binary
FROM ubuntu:24.04

# Some Ubuntu images have an ubuntu user - don't error if it doesn't exist
RUN userdel -r ubuntu || true

# We want jq and curl in the final image, but we don't need the support files
RUN apt-get update && \
	apt-get install -y jq curl && \
	apt-get clean && \
	rm -rf /usr/share/doc /usr/share/man /usr/share/zsh

RUN useradd -m -u 1000 -U -s /bin/sh -d /frequency frequency && \
	mkdir -p /chain-data /frequency/.local/share && \
	chown -R frequency:frequency /chain-data && \
	ln -s /chain-data /frequency/.local/share/frequency

# Copy the appropriate binary based on the target platform
COPY --from=base /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY linux/${TARGETARCH}/frequency /frequency/frequency
COPY scripts/healthcheck.sh /frequency/scripts/healthcheck.sh

# Set correct permissions and ownership
RUN chmod +x /frequency/frequency /frequency/scripts/healthcheck.sh && \
	chown -R frequency:frequency /frequency

USER frequency

# Frequency Chain Ports
# 9944 for Websocket and Rpc
# 30333 for P2P
# 9615 for Telemetry (prometheus)
# Relay Chain Ports
# 9945 for Websocket and Rpc
# 30334 for P2P
# 9616 for Telemetry (prometheus)
EXPOSE 9944 30333 9615 9945 30334 9616

HEALTHCHECK --interval=300s --timeout=75s --start-period=30s --retries=3 \
	CMD ["/frequency/scripts/healthcheck.sh"]

VOLUME ["/chain-data"]

ENTRYPOINT ["/frequency/frequency"]
