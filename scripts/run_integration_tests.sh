#!/usr/bin/env bash

function get_frequency_pid () {
    lsof -i tcp:9933 | grep frequency | xargs | awk '{print $2}'
}

function cleanup () {
    local signal="$1"

    case "$signal" in
        TERM|INT)
            # Catch TERM and INT signals and exit gracefully
            echo "Caught signal ${signal}; exiting..."
            exit
            ;;
        EXIT)
            # kill_freq.sh is not used here because we do not know what directory
            # the script is in when a signal is received. Therefore, we do not
            # know how to navigate to the kill_freq.sh script with relative paths.
            if [ -n "${PID}" ]
            then
                kill -9 ${PID}
                echo "Frequency has been killed. 💀"
            else
                echo "Frequency was not started by integration-test."
            fi
            ;;
    esac
}

RUNDIR=$(dirname ${0})
SKIP_JS_BUILD=
CHAIN="local_instant_sealing"

trap 'cleanup EXIT' EXIT
trap 'cleanup TERM' TERM
trap 'cleanup INT' INT

while getopts "sc:" OPTNAME
do
    case "${OPTNAME}" in
        "s")
            SKIP_JS_BUILD=1
        ;;
        "c")
            CHAIN=$OPTARG
        ;;
    esac
done
shift $((OPTIND-1))

case "${CHAIN}" in
    "local_instant_sealing")
        PROVIDER_URL="ws://127.0.0.1:9944"
        CHAIN_ENVIRONMENT="local"
        BLOCK_SEALING="instant"
        NPM_RUN_COMMAND="test"

        if [[ "$1" == "load" ]]; then
            NPM_RUN_COMMAND="test:load"
            BLOCK_SEALING="manual"
        fi
    ;;
    "local_relay")
        PROVIDER_URL="ws://127.0.0.1:9944"
        NPM_RUN_COMMAND="test:relay"
        CHAIN_ENVIRONMENT="local"
        BLOCK_SEALING="instant"
    ;;
    "frequency_rococo")
        PROVIDER_URL="wss://rpc.rococo.frequency.xyz"
        NPM_RUN_COMMAND="test:relay"
        CHAIN_ENVIRONMENT="rococo"
        BLOCK_SEALING="instant"
    ;;
esac

echo "The integration test output will be logged on this console"

echo "The Frequency node output will be logged to the file frequency.log."
echo "You can 'tail -f frequency.log' in another terminal to see both side-by-side."
echo ""
echo -e "Checking to see if Frequency is running..."

if [ -n "$( get_frequency_pid )" ]
then
    echo "Frequency is already running."
else
    echo "Building local Frequency executable..."
    if ! make build-local
    then
        echo "Error building Frequency executable; aborting."
        exit 1
    fi

    echo "Starting a Frequency Node with block sealing ${BLOCK_SEALING}..."
    case ${BLOCK_SEALING} in
        "instant") ${RUNDIR}/init.sh start-frequency-instant >& frequency.log &
        ;;
        "manual") ${RUNDIR}/init.sh start-frequency-manual >& frequency.log &
        ;;
    esac

    declare -i timeout_secs=60
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
fi

if [ "${SKIP_JS_BUILD}" = "1" ]
then
    echo "Skipping js/api-augment build"
else
    echo "Building js/api-augment..."
    cd js/api-augment
    npm i
    npm run fetch:local
    npm run --silent build
    cd dist
    echo "Packaging up into js/api-augment/dist/frequency-chain-api-augment-0.0.0.tgz"
    npm pack --silent
    cd ../../..
fi


cd integration-tests
echo "Installing js/api-augment/dist/frequency-chain-api-augment-0.0.0.tgz"
npm i ../js/api-augment/dist/frequency-chain-api-augment-0.0.0.tgz
npm install
echo "---------------------------------------------"
echo "Starting Tests..."
echo "---------------------------------------------"
set -x
CHAIN_ENVIRONMENT=$CHAIN_ENVIRONMENT WS_PROVIDER_URL="$PROVIDER_URL" npm run $NPM_RUN_COMMAND
