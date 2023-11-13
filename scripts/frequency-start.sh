#!/bin/sh

if [ -z "${SEALING_MODE}" ]
then
    SEALING_MODE=instant
fi

if [ -n "${SEALING_INTERVAL}" ]
then
    SEALING_INTERVAL="--sealing-interval=${SEALING_INTERVAL}"
fi

if [ "${CREATE_EMPTY_BLOCKS}" = true ]
then
    CREATE_EMPTY_BLOCKS="--sealing-create-empty-blocks"
fi

exec /frequency/frequency \
    --dev \
    -lruntime=debug \
    --no-telemetry \
	--no-prometheus \
	--port=30333 \
	--rpc-port=9944 \
	--rpc-external \
	--rpc-cors=all \
	--rpc-methods=Unsafe \
	--base-path=/data \
    --sealing=${SEALING_MODE} \
    ${SEALING_INTERVAL} \
    ${CREATE_EMPTY_BLOCKS} \
    $*
