#!/usr/bin/env bash

TEST="test"
START="start"

if [[ "$1" == "load" ]]; then
    TEST="test:load"
    START="start-manual"
fi

echo "The integration test output will be logged on this console"
echo "and the Frequency node output will be logged to the file frequency.log."
echo "You can 'tail -f frequency.log' in another terminal to see both side-by-side."
echo ""
echo -e "Checking to see if Frequency is running..."

PID=$(lsof -i tcp:9933 | grep frequency | grep -v grep | xargs | awk '{print $2}')

SHOULD_KILL=false

if [ -z "$PID" ]
then
    make build-local
    echo -e "Starting a Frequency Node..."
    make $START >& frequency.log &
    SHOULD_KILL=true
fi

while [ -z "$PID" ]
do
    sleep 3
    PID=$(lsof -i tcp:9933 | grep frequency | grep -v grep | xargs | awk '{print $2}')
    echo "Waiting for 9933 to be open..."
done

echo "---------------------------------------------"
echo "Frequency running here:"
echo "PID: ${PID}"
echo "---------------------------------------------"

echo "Building js/api-augment..."
cd js/api-augment
npm i
npm run fetch:local
npm run --silent build
cd dist
echo "Packaging up into js/api-augment/dist/frequency-chain-api-augment-0.0.0.tgz"
npm pack --silent
cd ../../..


cd integration-tests
echo "Installing js/api-augment/dist/frequency-chain-api-augment-0.0.0.tgz"
npm i ../js/api-augment/dist/frequency-chain-api-augment-0.0.0.tgz
npm install
echo "---------------------------------------------"
echo "Starting Tests..."
echo "---------------------------------------------"
WS_PROVIDER_URL="ws://127.0.0.1:9944" npm run $TEST

if $SHOULD_KILL
then
   pwd
   ../scripts/kill_freq.sh
fi
