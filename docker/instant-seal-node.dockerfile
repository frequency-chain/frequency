# Requires to run from repository root and to copy the binary in the build folder
FROM ubuntu:20.04 AS base
LABEL maintainer="Frequency Team"
LABEL description="Binary for Frequency parachain"

RUN apt-get update && apt-get install -y ca-certificates && update-ca-certificates

FROM ubuntu:20.04
RUN useradd -m -u 1000 -U -s /bin/sh -d /frequency frequency && \
	mkdir /data && \
	chown -R frequency:frequency /data

USER frequency

COPY --from=base /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --chown=frequency target/release/frequency /frequency/
RUN chmod uog+x /frequency/frequency

# 9933 P2P port
# 9944 for RPC call
# 30333 for Websocket
EXPOSE 9933 9944 30333

VOLUME ["/data"]

ENTRYPOINT ["/frequency/frequency"]
CMD [ "--dev", \
	"-lruntime=debug", \
	"--instant-sealing", \
	"--wasm-execution=compiled", \
	"--execution=wasm", \
	"--no-telemetry", \
	"--no-prometheus", \
	"--port=30333", \
	"--rpc-port=9933", \
	"--ws-port=9944", \
	"--rpc-external", \
	"--rpc-cors=all", \
	"--ws-external", \
	"--rpc-methods=Unsafe", \
	"--tmp" \
	]
