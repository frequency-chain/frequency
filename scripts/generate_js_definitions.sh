#!/bin/zsh

# ! Please make sure to have both docker and docker-compose installed

set -e

STEP_COLOR='\033[4;36m'    # Cyan Underline
BOLD_MESSAGE='\033[1;33m'  # Bold Yellow
SUCCESS='\033[0;92m'       # Green
NC='\033[0m'               # No Color

echo "\n${STEP_COLOR}Starting the Frequency container...${NC}"

make start > /dev/null 2>&1 &

PID=$(lsof -i tcp:9933 | grep frequency | grep -v grep | xargs | awk '{print $2}')

# Poll for PID

while [ -z "$PID" ]
do
    PID=$(lsof -i tcp:9933 | grep frequency | grep -v grep | xargs | awk '{print $2}')
done

echo "${SUCCESS}"
echo "---------------------------------------------"
echo "Use this PID to kill the Frequency process:"
echo "${BOLD_MESSAGE}PID: ${PID}${SUCCESS}"
echo "---------------------------------------------"
echo "${NC}"

# cd into js dir
cd js/api-augment

npm run fetch:local && npm run build
