FROM ubuntu:20.04
LABEL maintainer="MRC Team"
LABEL description="Create an image with MRC binary built in main."

WORKDIR /mrc

RUN apt-get update && \
    apt-get install -y apt-utils apt-transport-https software-properties-common readline-common curl vim wget gnupg gnupg2 gnupg-agent ca-certificates tini

COPY mrc_binary/mrc-collator /mrc/target/release/

# Checks
RUN ldd /usr/local/bin/mrc-collator && \
	/usr/local/bin/mrc-collator --version

# Add chain resources to image
COPY res /res/

COPY scripts /scripts/

RUN chmod +x ./scripts/run_collator.sh
RUN chmod +x ./scripts/init.sh

ENV MRC_BINARY_PATH=/mrc/target/release/mrc-collator

# USER mrc # see above

VOLUME ["/data"]

ENTRYPOINT ["/usr/bin/tini", "--"]

CMD ["/bin/bash", "./scripts/init.sh", "start-mrc-container"]

