#!/bin/bash

set -e

PID=$(lsof -i tcp:9944 | grep frequency | xargs | awk '{print $2}')

if [ -n "${PID}" ]
then
    kill -9 ${PID}
    echo "Frequency has been killed. 💀"
fi
