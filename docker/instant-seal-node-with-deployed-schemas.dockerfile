# Docker image for running Frequency parachain node container (with collating)
# locally in instant seal mode then deploying schemas to that node.
# Requires to run from repository root and to copy
# the binary in the build folder.

#This pulls the latest instant-seal-node image
FROM frequencychain/instant-seal-node as frequency-image

#Creates base image
FROM --platform=linux/amd64 ubuntu:20.04 AS base

LABEL maintainer="Frequency"
LABEL description="Frequency collator node in instant seal mode"

RUN apt-get update && apt-get install -y ca-certificates && update-ca-certificates

#Install git and clone schemas repo
RUN apt-get update && apt-get install -y git

RUN git clone https://github.com/LibertyDSNP/schemas.git

# Install node-js to base image
RUN apt-get update && apt-get install -y curl gnupg
RUN curl -sL https://deb.nodesource.com/setup_16.x | bash
RUN apt-get update && apt-get install -y nodejs

# Add frequency user

RUN useradd -m -u 1000 -U -s /bin/sh -d /frequency frequency && \
    chown -R frequency:frequency /schemas  && \
    mkdir -p /data /frequency/.local/share && \
	chown -R frequency:frequency /data && \
	ln -s /data /frequency/.local/share/frequency

USER frequency

# Copy over depoly_schemas script to base image
COPY --chown=frequency scripts/deploy_schemas.sh ./frequency/
RUN chmod +x ./frequency/deploy_schemas.sh

# Copy over latest frequency binary from instant-seal image to base image
COPY --from=frequency-image --chown=frequency /frequency/frequency /frequency/frequency
RUN chmod +x /frequency/frequency

# 9933 P2P port
# 9944 for RPC call
# 30333 for Websocket
EXPOSE 9933 9944 30333

VOLUME ["/data"]

# Params which can be overriden from CLI
CMD ["/bin/bash", "frequency/deploy_schemas.sh"]
