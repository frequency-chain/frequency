#!/usr/bin/env bash

set -e

node=$1

if [[ "$node" == "alice" || "$node" == "bob" ]]; then
   docker logs -f $node
else
   echo -e "\033[31m Usage: logs.sh alice/bob"
fi


