#!/bin/bash

set -x

if [[ -z "$1" || -z $2 || -z $3 ]]; then
  echo "usage: $0 'collator sudo secret' \$ws_provider \$wasm_location"
  # fx: $0 'collator sudo secret' wss://dev.net.t3rn.io /tmp/wasm
  exit 1
fi


echo "🏭 installing subwasm..."
cargo install --locked --git https://github.com/chevdor/subwasm

sudo_secret=$1
ws_provider=$2
wasm_location=$3

hash=$(subwasm info --json $wasm_location | jq -r .blake2_256)

cd scripts/js/onboard

npm i && npm run upgrade-auth $ws_provider $sudo_secret $hash
