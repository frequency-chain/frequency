#!/usr/bin/env bash

function get_frequency_pid () {
    lsof -i tcp:9944 | grep frequency | xargs | awk '{print $2}'
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
                echo "Frequency was not started by this script."
            fi
            ;;
    esac
}

RUNDIR=$(dirname ${0})
SKIP_JS_BUILD=
CHAIN="development"

# A distinction is made between the local node and the the test chain
# because the local node will be built and generate the js api augment
# for the polkadot.js api even when testing against a live chain.
LOCAL_NODE_BLOCK_SEALING="instant"

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
    "development")
        PROVIDER_URL="ws://127.0.0.1:9944"
        NPM_RUN_COMMAND="test"
        CHAIN_ENVIRONMENT="dev"

        if [[ "$1" == "load" ]]; then
            NPM_RUN_COMMAND="test:load"
            LOCAL_NODE_BLOCK_SEALING="manual"
        fi
    ;;
    "serial")
        PROVIDER_URL="ws://127.0.0.1:9944"
        NPM_RUN_COMMAND="test:serial"
        CHAIN_ENVIRONMENT="dev"
    ;;
    "paseo_local")
        PROVIDER_URL="ws://127.0.0.1:9944"
        NPM_RUN_COMMAND="test:relay"
        CHAIN_ENVIRONMENT="paseo-local"
    ;;
    "paseo_testnet")
        PROVIDER_URL="wss://0.rpc.amplica.io"
        NPM_RUN_COMMAND="test:relay"
        CHAIN_ENVIRONMENT="paseo-testnet"

        read -p "Enter the seed phrase for the Frequency Paseo account funding source: " FUNDING_ACCOUNT_SEED_PHRASE
    ;;
esac

echo "The E2E test output will be logged on this console"

echo "The Frequency node output will be logged to the file frequency.log."
echo "You can 'tail -f frequency.log' in another terminal to see both side-by-side."
echo ""
echo -e "Checking to see if Frequency is running..."

if [ -n "$( get_frequency_pid )" ]
then
    echo "Frequency is already running."
else
    if [ "${CHAIN_ENVIRONMENT}" = "paseo-local" ]
    then
        echo "Frequency is not running."
        echo "The intended use case of running E2E tests with a chain environment"
        echo "of \"paseo-local\" is to run the tests against a locally running Frequency"
        echo "chain with locally running Polkadot relay nodes."
        exit 1
    fi

    echo "Building a no-relay Frequency executable..."
    if ! make build-no-relay
    then
        echo "Error building Frequency executable; aborting."
        exit 1
    fi

    echo "Starting a Frequency Node with block sealing ${LOCAL_NODE_BLOCK_SEALING}..."
    case ${LOCAL_NODE_BLOCK_SEALING} in
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


cd e2e
echo "Installing js/api-augment/dist/frequency-chain-api-augment-0.0.0.tgz"
npm i ../js/api-augment/dist/frequency-chain-api-augment-0.0.0.tgz
npm install
echo "---------------------------------------------"
echo "Starting Tests..."
echo "---------------------------------------------"

CHAIN_ENVIRONMENT=$CHAIN_ENVIRONMENT FUNDING_ACCOUNT_SEED_PHRASE=$FUNDING_ACCOUNT_SEED_PHRASE WS_PROVIDER_URL="$PROVIDER_URL" npm run $NPM_RUN_COMMAND
