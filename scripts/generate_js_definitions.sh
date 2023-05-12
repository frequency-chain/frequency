#!/usr/bin/env bash

set -e

STEP_COLOR='\033[4;36m'    # Cyan Underline
MESSAGE='\033[0;33m'       # Yellow
BOLD_MESSAGE='\033[1;33m'  # Bold Yellow
SUCCESS='\033[0;92m'       # Green
NC='\033[0m'               # No Color

echo -e "${STEP_COLOR}Checking to see if Frequency is running...${NC}"
echo ""
echo -e "${MESSAGE}Is Frequency running?${NC}"

PID=$(lsof -i tcp:9933 | grep frequency | grep -v grep | xargs | awk '{print $2}')

if [ -z "$PID" ]
then
    echo -e "${MESSAGE}"
    echo "No."
    echo -e "${NC}"
    echo -e "${STEP_COLOR}Generating using CLI...${NC}"
    rm -f ./js/api-augment/metadata.json
    cargo run --features frequency-rococo-local -- export-metadata --tmp --chain=frequency-rococo-local ./js/api-augment/metadata.json
    # cd into js dir
    cd "js/api-augment"
    npm install # in case things have changed
    npm run build
else
    echo -e "${SUCCESS}"
    echo "Yes. ( You better go catch it ;-) )"
    echo ""
    echo "---------------------------------------------"
    echo "Use this PID to kill the Frequency process:"
    echo -e "${BOLD_MESSAGE}PID: ${PID}${SUCCESS}"
    echo "---------------------------------------------"
    echo -e "${NC}"

    # cd into js dir
    cd "js/api-augment"
    npm install # in case things have changed
    npm run fetch:local
    npm run build
fi

# Generate the new packed tgz
cd dist
npm pack
