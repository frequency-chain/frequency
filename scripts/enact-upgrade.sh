#!/bin/bash

set -x

if [[ -z "$1" || -z $2 || -z $3 ]]; then
  echo "usage: $0 'collator sudo secret' \$ws_provider \$wasm_location"
  # fx: $0 'collator sudo secret' wss://dev.net.t3rn.io /tmp/wasm
  exit 1
fi

sudo_secret=$1
ws_provider=$2
wasm_location=$3

cd scripts/js/onboard

npm i && npm run upgrade-enact $ws_provider $sudo_secret $wasm_location
