#!/bin/zsh

set -e

PID="$(ps aux | grep target/release/frequency | grep -v grep | xargs | awk '{print $2}')"

if ! [ -z $PID ]
then
    kill -9 $PID
    echo "Frequency is dead."
fi
