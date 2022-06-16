FROM --platform=linux/amd64 ubuntu:20.04
LABEL maintainer="MRC Team"
LABEL description="Create an image with MRC binary built in main."

WORKDIR /mrc

RUN apt-get update && \
    apt-get install -y jq apt-utils apt-transport-https software-properties-common readline-common curl vim wget gnupg gnupg2 gnupg-agent ca-certificates tini && \
    rm -rf /var/lib/apt/lists/*

COPY target/release/mrc-collator ./target/release/
RUN chmod +x target/release/mrc-collator

RUN ls ./target/release

# Checks
RUN ls -lah /
RUN file ./target/release/mrc-collator && \
    ./target/release/mrc-collator --version

# Add chain resources to image
COPY res ./res/

COPY scripts ./scripts/

RUN chmod +x ./scripts/run_collator.sh
RUN chmod +x ./scripts/init.sh
RUN chmod +x ./scripts/healthcheck.sh

ENV MRC_BINARY_PATH=./target/release/mrc-collator

HEALTHCHECK --interval=300s --timeout=75s --start-period=30s --retries=3 \
    CMD ["./scripts/healthcheck.sh"]

VOLUME ["/data"]

ENTRYPOINT ["/usr/bin/tini", "--"]

CMD ["/bin/bash", "./scripts/init.sh", "start-mrc-container"]

# 9933 p2p port
# 9944 rpc port
# 30333 ws port
EXPOSE 9933 9944 30333
