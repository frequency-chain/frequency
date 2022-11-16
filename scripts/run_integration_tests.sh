#!/usr/bin/env bash

set -e

echo -e "Checking to see if Frequency is running..."

PID=$(lsof -i tcp:9933 | grep frequency | grep -v grep | xargs | awk '{print $2}')

if [ -z "$PID" ]
then
    echo -e "Starting a Frequency Node..."
    make start &
fi

while [ -z "$PID" ]
do
    PID=$(ps aux | grep target/release/frequency | grep -v grep | xargs | awk '{print $2}')
done

echo "---------------------------------------------"
echo "Frequency running here:"
echo "PID: ${PID}"
echo "---------------------------------------------"

cd integration-tests
npm i
WS_PROVIDER_URL="ws://127.0.0.1:9944" npm test

kill -9 $PID > /dev/null
echo "Frequency node has been stopped"