# Docker image for running Frequency parachain node container (with collating)
# locally as a standalone node. Multi-architecture support for amd64 and arm64
FROM ubuntu:24.04 AS base
LABEL maintainer="Frequency"
LABEL description="Frequency standalone node"

RUN apt-get update && apt-get install -y ca-certificates && update-ca-certificates

# This is the 2nd stage: a very small image where we copy the Frequency binary
FROM ubuntu:24.04

# From QEMU
ARG TARGETARCH

# Some Ubuntu images have an ubuntu user
RUN userdel -r ubuntu || true

# We want jq and curl in the final image, but we don't need the support files
RUN apt-get update && \
	apt-get install -y jq curl && \
	apt-get clean && \
	rm -rf /usr/share/doc /usr/share/man /usr/share/zsh

RUN useradd -m -u 1000 -U -s /bin/sh -d /frequency frequency && \
	mkdir -p /data /frequency/.local/share && \
	chown -R frequency:frequency /data && \
	ln -s /data /frequency/.local/share/frequency

# For local testing only
# COPY --chown=frequency target/x86_64-unknown-linux-gnu/debug/frequency ./frequency/frequency
# Copy the appropriate binary based on the target platform
COPY --from=base /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY linux/${TARGETARCH}/frequency /frequency/frequency
COPY scripts/frequency-start.sh /frequency/frequency-start.sh
COPY scripts/healthcheck.sh /frequency/scripts/healthcheck.sh

# Set correct permissions and ownership BEFORE switching user
RUN chmod +x /frequency/frequency /frequency/frequency-start.sh /frequency/scripts/healthcheck.sh && \
	chown -R frequency:frequency /frequency

# Switch to non-root user after setting permissions
USER frequency

# 9944 for RPC call
EXPOSE 9944

HEALTHCHECK --interval=300s --timeout=75s --start-period=30s --retries=3 \
	CMD ["/frequency/scripts/healthcheck.sh"]

VOLUME ["/data"]

ENTRYPOINT ["/frequency/frequency-start.sh"]
