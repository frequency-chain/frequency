FROM ubuntu:20.04
LABEL maintainer="MRC Team"
LABEL description="Create an image with MRC binary built in main."

WORKDIR /mrc

RUN apt-get update && \
    apt-get install -y apt-utils apt-transport-https software-properties-common readline-common curl vim wget gnupg gnupg2 gnupg-agent ca-certificates tini

RUN apt-get install jq -y

COPY ./target/release/mrc-collator ./target/release/

RUN ls ./target/release

# Checks
RUN ldd ./target/release/mrc-collator && \
    ./target/release/mrc-collator --version

# Add chain resources to image
COPY res ./res/

COPY scripts ./scripts/

RUN chmod +x ./scripts/run_collator.sh
RUN chmod +x ./scripts/init.sh

ENV MRC_BINARY_PATH=./target/release/mrc-collator

VOLUME ["/data"]

ENTRYPOINT ["/usr/bin/tini", "--"]

CMD ["/bin/bash", "./scripts/init.sh", "start-mrc-container"]
