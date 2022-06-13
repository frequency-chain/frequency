FROM phusion/baseimage:focal-1.0.0
LABEL maintainer="MRC Team"
LABEL description="Create an image with MRC binary built in main."

RUN apt-get update && \
	apt-get dist-upgrade -y -o Dpkg::Options::="--force-confold" && \
	apt-get install -y cmake pkg-config libssl-dev git clang libclang-dev

COPY mrc_binary/mrc-collator /usr/local/bin

# Checks
RUN ldd /usr/local/bin/mrc-collator && \
	/usr/local/bin/mrc-collator --version

# Add chain resources to image
COPY res /res/

# USER mrc # see above

VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/mrc-collator"]
