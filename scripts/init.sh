#!/usr/bin/env bash

set -e

cmd=$1
chain_spec="${RAW_PARACHAIN_CHAIN_SPEC:-./res/genesis/local/rococo-local-frequency-2000-raw.json}"
# The runtime we want to use
parachain="${PARA_CHAIN_CONFIG:-rococo-2000}"
# The parachain Id we want to use
para_id="${PARA_ID:-2000}"
# The tmp base directory
base_dir=/tmp/frequency
# Option to use the Docker image to export state & wasm
docker_onboard="${DOCKER_ONBOARD:-false}"
frequency_docker_image_tag="${PARA_DOCKER_IMAGE_TAG:-frequency-latest}"

chain="${RELAY_CHAIN_SPEC:-./res/rococo-local.json}"

case $cmd in
install-toolchain)
  ./scripts/install_toolchain.sh
  ;;

start-relay-chain)
  echo "Starting local relay chain with Alice and Bob..."
  docker-compose up -d relay_alice relay_bob
  ;;

stop-relay-chain)
  echo "Stopping relay chain..."
  docker-compose down
  ;;

start-frequency-docker)
  echo "Starting frequency container with Alice..."
  docker-compose up --build collator_frequency
  ;;

stop-frequency-docker)
  echo "Stopping frequency container with Alice..."
  docker-compose down collator_frequency
  ;;

start-frequency)
  printf "\nBuilding frequency with runtime '$parachain' and id '$para_id'...\n"
  cargo build --release

  parachain_dir=$base_dir/parachain/${para_id}
  mkdir -p $parachain_dir;

  if [ "$2" == "purge" ]; then
    echo "purging parachain..."
    rm -rf $parachain_dir
  fi

  ./scripts/run_collator.sh \
    --chain="${chain_spec}" --alice \
    --base-path=$parachain_dir/data \
    --wasm-execution=compiled \
    --execution=wasm \
    --force-authoring \
    --port $((30333)) \
    --rpc-port $((9933)) \
    --ws-port $((9944)) \
    --rpc-external \
    --rpc-cors all \
    --ws-external \
    --rpc-methods=Unsafe \
    --state-cache-size 0 \
  ;;

start-frequency-instant)
  printf "\nBuilding frequency with runtime instant sealing ...\n"
  cargo build --release

  parachain_dir=$base_dir/parachain/${para_id}
  mkdir -p $parachain_dir;

  if [ "$2" == "purge" ]; then
    echo "purging parachain..."
    rm -rf $parachain_dir
  fi

  ./target/release/frequency \
    --dev \
    -lruntime=debug \
    --instant-sealing \
    --wasm-execution=compiled \
    --execution=wasm \
    --no-telemetry \
    --no-prometheus \
    --port $((30333)) \
    --rpc-port $((9933)) \
    --ws-port $((9944)) \
    --rpc-external \
    --rpc-cors all \
    --ws-external \
    --rpc-methods=Unsafe \
    --tmp
  ;;

start-frequency-container)

  parachain_dir=$base_dir/parachain/${para_id}
  mkdir -p $parachain_dir;
  frequency_default_port=$((30333))
  frequency_default_rpc_port=$((9933))
  frequency_default_ws_port=$((9944))
  frequency_port="${Frequency_PORT:-$frequency_default_port}"
  frequency_rpc_port="${Frequency_RPC_PORT:-$frequency_default_rpc_port}"
  frequency_ws_port="${Frequency_WS_PORT:-$frequency_default_ws_port}"

  ./scripts/run_collator.sh \
    --chain="${chain_spec}" --alice \
    --base-path=$parachain_dir/data \
    --wasm-execution=compiled \
    --execution=wasm \
    --force-authoring \
    --port "${frequency_port}" \
    --rpc-port "${frequency_rpc_port}" \
    --ws-port "${frequency_ws_port}" \
    --rpc-external \
    --rpc-cors all \
    --ws-external \
    --rpc-methods=Unsafe \
    --state-cache-size 0 \
  ;;

register-frequency)
  echo "reserving and registering parachain with relay via first available slot..."

  cd scripts/js/onboard
  yarn && yarn register "ws://0.0.0.0:9946" "//Alice"
  ;;

onboard-frequency)
  echo "Onboarding parachain with runtime '$parachain' and id '$para_id'..."

   onboard_dir="$base_dir/onboard"
   mkdir -p $onboard_dir

   wasm_location="$onboard_dir/${parachain}-${para_id}.wasm"
    if [ "$docker_onboard" == "true" ]; then
      genesis=$(docker run -it {REPO_NAME}/frequency:${frequency_docker_image_tag} export-genesis-state --chain="${chain_spec}")
      docker run -it {REPO_NAME}/frequency:${frequency_docker_image_tag} export-genesis-wasm --chain="${chain_spec}" > $wasm_location
    else
      genesis=$(./target/release/frequency export-genesis-state --chain="${chain_spec}")
      ./target/release/frequency export-genesis-wasm --chain="${chain_spec}" > $wasm_location
    fi

  echo "WASM path:" "${parachain}-${para_id}.wasm"

  cd scripts/js/onboard
  yarn && yarn onboard "ws://0.0.0.0:9946" "//Alice" ${para_id} "${genesis}" $wasm_location
  ;;

offboard-frequency)
  echo "cleaning up parachain for id '$para_id'..."

  cd scripts/js/onboard
  yarn && yarn cleanup "ws://0.0.0.0:9946" "//Alice" ${para_id}
  ;;
esac
