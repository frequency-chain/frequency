#!/usr/bin/env bash

function get_frequency_pid () {
    lsof -i tcp:9933 | grep frequency | xargs | awk '{print $2}'
}

function cleanup () {
    if ${SHOULD_KILL}
    then
        ${RUNDIR}/kill_freq.sh
    fi
}

RUNDIR=$(dirname ${0})
SKIP_JS_BUILD=
trap cleanup EXIT KILL INT

while getopts "s" OPTNAME
do
    case ${OPTNAME} in
        "s") SKIP_JS_BUILD=1
        ;;
    esac
done

echo "The integration test output will be logged on this console"
echo "and the Frequency node output will be logged to the file frequency.log."
echo "You can 'tail -f frequency.log' in another terminal to see both side-by-side."
echo ""
echo -e "Checking to see if Frequency is running..."

PID=$( get_frequency_pid )

SHOULD_KILL=false

if [ -z "${PID}" ]
then
    echo "Building local Frequency executable..."
    if ! make build-local
    then
        echo "Error building Frequency executable; aborting."
        exit 1
    fi
    echo "Starting a Frequency Node..."
    ${RUNDIR}/init.sh start-frequency-instant >& frequency.log &
    SHOULD_KILL=true
fi

declare -i timeout_secs=30
declare -i i=0
while (( !PID && i < timeout_secs ))
do
   PID=$( get_frequency_pid )
   sleep 1
   (( i += 1 ))
done

if [ -z "${PID}" ]
then
    echo "Unable to find or start a Frequency node; aborting."
    exit 1
fi

echo "---------------------------------------------"
echo "Frequency running here:"
echo "PID: ${PID}"
echo "---------------------------------------------"

if [ -z "${SKIP_JS_BUILD}" ]
then
    echo "Building js/api-augment..."
    ( cd js/api-augment ;\
    npm i ;\
    npm run fetch:local ;\
    npm run --silent build ;\
    cd dist ;\
    echo "Packaging up into js/api-augment/dist/frequency-chain-api-augment-0.0.0.tgz" ;\
    npm pack --silent )


    ( cd integration-tests ;\
    echo "Installing js/api-augment/dist/frequency-chain-api-augment-0.0.0.tgz" ;\
    npm i ../js/api-augment/dist/frequency-chain-api-augment-0.0.0.tgz ; )
fi

( cd integration-tests ;\
    npm install ;\
    echo "---------------------------------------------" ;\
    echo "Starting Tests..." ;\
    echo "---------------------------------------------" ;\
    WS_PROVIDER_URL="ws://127.0.0.1:9944" npm test )
