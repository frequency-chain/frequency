# Docker image for running Frequency parachain node container (with collating)
# locally as a standalone node. Requires to run from repository root and to copy
# the binary in the build folder.
# This is the build stage for Polkadot. Here we create the binary in a temporary image.
FROM --platform=linux/amd64 ubuntu:24.04 AS base

LABEL maintainer="Frequency"
LABEL description="Frequency standalone node"

RUN apt-get update && apt-get install -y ca-certificates && update-ca-certificates

# This is the 2nd stage: a very small image where we copy the Frequency binary
FROM --platform=linux/amd64 ubuntu:24.04

# Some Ubuntu images have an ubuntu user
RUN userdel -r ubuntu

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
COPY --chown=frequency target/release/frequency ./frequency/
COPY --chown=frequency docker/frequency-start.sh ./frequency/
RUN chmod +x ./frequency/frequency ./frequency/frequency-start.sh

# 9944 for RPC call
EXPOSE 9944

HEALTHCHECK --interval=300s --timeout=75s --start-period=30s --retries=3 \
	CMD ["./scripts/healthcheck.sh"]

VOLUME ["/data"]

ENTRYPOINT [ "/frequency/frequency-start.sh" ]
