#!/bin/bash

if [[ -z "$1" ]]; then
  cat <<-EOF
Usage: $0 <websocket.node>,
  where:
    <websocket.node> is a node running on websocket port [wss|ws://websocket_provider:port]
      For example ws://127.0.0.1:9944
EOF

  exit 1
fi


echo "checking for 🏭 subwasm and installing if needed..."
which subwasm || cargo install --locked --git https://github.com/chevdor/subwasm --tag v0.19.1

ws_provider=$1

pallets=$(subwasm metadata "${ws_provider}")

echo "----Pallets----"
echo "${pallets}"
echo ""

regex_for_pallet='^ - .*'
while IFS= read -r pallet; do
  if [[ $pallet =~ $regex_for_pallet ]];
  then
    pallet_name=$(echo $pallet | cut -d " " -f 3)
    echo "----Pallet ${pallet_name}----"
    pallet_data=$(subwasm metadata -m "${pallet_name}" "${ws_provider}")
    echo -e "${pallet_data}"
    echo ""
  fi
done <<< "$pallets"
