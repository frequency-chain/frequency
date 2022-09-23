#!/bin/zsh

set -e

STEP_COLOR='\033[4;36m'    # Cyan Underline
MESSAGE='\033[0;33m'       # Yellow
BOLD_MESSAGE='\033[1;33m'  # Bold Yellow
SUCCESS='\033[0;92m'       # Green
NC='\033[0m'               # No Color

echo "${STEP_COLOR}Checking to see if Frequency is running...${NC}"
echo ""
echo "${MESSAGE}Is Frequency running?${NC}"

PID=$(lsof -i tcp:9933 | grep frequency | grep -v grep | xargs | awk '{print $2}')

if [ -z "$PID" ]
then
    echo "${MESSAGE}"
    echo "No."
    echo "${NC}"
    echo "${STEP_COLOR}Starting Frequency in Instant Sealing Mode...${NC}"
    make start > /dev/null 2>&1 &

    # Poll for PID
    while [ -z "$PID" ]
    do
        PID=$(lsof -i tcp:9933 | grep frequency | grep -v grep | xargs | awk '{print $2}')
    done
else
    echo "${SUCCESS}"
    echo "Yes. ( You better go catch it ;-) )"
fi
echo ""
echo "---------------------------------------------"
echo "Use this PID to kill the Frequency process:"
echo "${BOLD_MESSAGE}PID: ${PID}${SUCCESS}"
echo "---------------------------------------------"
echo "${NC}"

# cd into js dir
cd "js/api-augment"

npm run fetch:local && npm run build
