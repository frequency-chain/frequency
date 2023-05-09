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
chain="${RELAY_CHAIN_SPEC:-./resources/rococo-local.json}"

case $cmd in

start-relay-chain)
  echo "Starting local relay chain with Alice and Bob..."
  cd docker
  docker-compose up -d relay_alice relay_bob
  ;;

stop-relay-chain)
  echo "Stopping relay chain..."
  cd docker
  docker-compose down
  ;;

start-frequency-docker)
  echo "Starting frequency container with Alice..."
  cd docker
  docker-compose up --build collator_frequency
  ;;

stop-frequency-docker)
  echo "Stopping frequency container with Alice..."
  cd docker
  docker-compose down
  ;;

start-frequency)
  printf "\nBuilding frequency with runtime '$parachain' and id '$para_id'...\n"
  cargo build --release --features frequency-rococo-local

  parachain_dir=$base_dir/parachain/${para_id}
  mkdir -p $parachain_dir;

  if [ "$2" == "purge" ]; then
    echo "purging parachain..."
    rm -rf $parachain_dir
  fi

  ./scripts/run_collator.sh \
    --chain="frequency-local" --alice \
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
  printf "\nBuilding Frequency with runtime instant sealing ...\n"
  cargo build --features frequency-no-relay

  parachain_dir=$base_dir/parachain/${para_id}
  mkdir -p $parachain_dir;

  if [ "$2" == "purge" ]; then
    echo "purging parachain..."
    rm -rf $parachain_dir
  fi

  ./target/debug/frequency \
    --dev \
    -lruntime=debug \
    --instant-sealing \
    --wasm-execution=compiled \
    --execution=native \
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

start-frequency-manual)
  printf "\nBuilding frequency with runtime manual sealing ...\n"
  cargo build --locked --features frequency-no-relay

  parachain_dir=$base_dir/parachain/${para_id}
  mkdir -p $parachain_dir;

  if [ "$2" == "purge" ]; then
    echo "purging parachain..."
    rm -rf $parachain_dir
  fi

  echo "---------------------------------------"
  echo "Running Frequency in manual seal mode."
  echo "Run 'make local-block' to seal a block."
  echo "---------------------------------------"

  ./target/debug/frequency \
    --dev \
    -lruntime=debug \
    --manual-sealing \
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
    --chain="frequency-local" --alice \
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

register-frequency-rococo-local)
  echo "reserving and registering parachain with relay via first available slot..."

  cd scripts/js/onboard
  yarn && yarn register "ws://0.0.0.0:9946" "//Alice"
  ;;

onboard-frequency-rococo-local)
  echo "Onboarding parachain with runtime '$parachain' and id '$para_id'..."

   onboard_dir="$base_dir/onboard"
   mkdir -p $onboard_dir

   wasm_location="$onboard_dir/${parachain}-${para_id}.wasm"
    if [ "$docker_onboard" == "true" ]; then
      genesis=$(docker run -it {REPO_NAME}/frequency:${frequency_docker_image_tag} export-genesis-state --chain="frequency-local")
      docker run -it {REPO_NAME}/frequency:${frequency_docker_image_tag} export-genesis-wasm --chain="frequency-local" > $wasm_location
    else
      genesis=$(./target/release/frequency export-genesis-state --chain="frequency-local")
      ./target/release/frequency export-genesis-wasm --chain="frequency-local" > $wasm_location
    fi

  echo "WASM path:" "${wasm_location}"

  cd scripts/js/onboard
  yarn && yarn onboard "ws://0.0.0.0:9946" "//Alice" ${para_id} "${genesis}" $wasm_location
  ;;

offboard-frequency-rococo-local)
  echo "cleaning up parachain for id '$para_id'..."

  cd scripts/js/onboard
  yarn && yarn cleanup "ws://0.0.0.0:9946" "//Alice" ${para_id}
  ;;

upgrade-frequency-rococo-local)

  root_dir=$(git rev-parse --show-toplevel)
  echo "root_dir is set to $root_dir"

  # Due to defaults and profile=release, the target directory will be $root_dir/target/release
  cargo build \
    --locked \
    --profile release \
    --package frequency-runtime \
    --features frequency-rococo-local \
    -Z unstable-options

  wasm_location=$root_dir/target/release/wbuild/frequency-runtime/frequency_runtime.compact.compressed.wasm

  ./scripts/runtime-upgrade.sh "//Alice" "ws://0.0.0.0:9944" $wasm_location

  ./scripts/enact-upgrade.sh "//Alice" "ws://0.0.0.0:9944" $wasm_location

  ;;

upgrade-frequency-no-relay)

  root_dir=$(git rev-parse --show-toplevel)
  echo "root_dir is set to $root_dir"

  # Due to defaults and profile=release, the target directory will be $root_dir/target/release
  cargo build \
    --locked \
    --profile release \
    --package frequency-runtime \
    --features frequency-no-relay \
    -Z unstable-options

  wasm_location=$root_dir/target/release/wbuild/frequency-runtime/frequency_runtime.compact.compressed.wasm

  ./scripts/runtime-upgrade.sh "//Alice" "ws://0.0.0.0:9944" $wasm_location

  ./scripts/enact-upgrade.sh "//Alice" "ws://0.0.0.0:9944" $wasm_location

  ;;

esac
