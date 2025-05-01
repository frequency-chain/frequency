# Docker image for running Frequency parachain node container (with collating)
# locally as a standalone node. Multi-architecture support for amd64 and arm64
FROM --platform=$TARGETPLATFORM ubuntu:24.04 AS base
LABEL maintainer="Frequency"
LABEL description="Frequency standalone node"

RUN apt-get update && apt-get install -y ca-certificates && update-ca-certificates

# This is the 2nd stage: a very small image where we copy the Frequency binary
FROM --platform=$TARGETPLATFORM ubuntu:24.04

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

USER frequency

COPY --from=base /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
# For local testing only
# COPY --chown=frequency target/x86_64-unknown-linux-gnu/debug/frequency ./frequency/frequency
# Copy the appropriate binary based on the target platform
COPY linux/${TARGETARCH}/frequency /frequency/frequency
COPY scripts/frequency-start.sh /frequency/frequency-start.sh
COPY scripts/healthcheck.sh /frequency/scripts/healthcheck.sh

RUN chmod +x /frequency/frequency /frequency/frequency-start.sh

# 9944 for RPC call
EXPOSE 9944

HEALTHCHECK --interval=300s --timeout=75s --start-period=30s --retries=3 \
	CMD ["/frequency/scripts/healthcheck.sh"]

VOLUME ["/data"]

ENTRYPOINT ["/frequency/frequency-start.sh"]
