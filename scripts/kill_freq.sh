#!/bin/zsh

set -e

PID=$(lsof -i tcp:9933 | grep frequency | grep -v grep | xargs | awk '{print $2}')

if ! [ -z $PID ]
then
    kill -9 $PID
    echo "Frequency has been killed. 💀"
fi
