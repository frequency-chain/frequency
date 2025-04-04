#!/usr/bin/env bash

set -e

cmd=$1
chain_spec="${RAW_PARACHAIN_CHAIN_SPEC:-./res/genesis/local/paseo-local-frequency-2000-raw.json}"
# The runtime we want to use
parachain="${PARA_CHAIN_CONFIG:-paseo-2000}"
# The parachain Id we want to use
para_id="${PARA_ID:-2000}"
# The tmp base directory
base_dir=/tmp/frequency
# Option to use the Docker image to export state & wasm
docker_onboard="${DOCKER_ONBOARD:-false}"
frequency_docker_image_tag="${PARA_DOCKER_IMAGE_TAG:-frequency-latest}"
chain="${RELAY_CHAIN_SPEC:-./resources/paseo-local.json}"
# offchain options
offchain_params="--offchain-worker=never"

if [ "$2" == "with-offchain" ]; then
  offchain_params="--offchain-worker=always --enable-offchain-indexing=true"
fi


case $cmd in

start-paseo-relay-chain)
  echo "Starting local relay chain with Alice and Bob..."
  cd docker
  docker-compose up -d relay_paseo_alice relay_paseo_bob
  ;;

stop-paseo-relay-chain)
  echo "Stopping paseo chain..."
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

start-paseo-collator-alice)
  printf "\nBuilding frequency with runtime '$parachain' and id '$para_id'...\n"
  cargo build --release --features frequency-local

  parachain_dir_alice=$base_dir/parachain/alice/${para_id}
  mkdir -p $parachain_dir_alice;

  if [ "$2" == "purge" ]; then
    echo "purging parachain..."
    rm -rf $parachain_dir_alice
  fi

  "${Frequency_BINARY_PATH:-./target/release/frequency}" key generate-node-key --base-path=$parachain_dir_alice/data

  ./scripts/run_collator.sh \
    --chain="frequency-paseo-local" --alice \
    --base-path=$parachain_dir_alice/data \
    --wasm-execution=compiled \
    --force-authoring \
    --port $((30333)) \
    --rpc-port $((9944)) \
    --rpc-external \
    --rpc-cors all \
    --rpc-methods=Unsafe \
    --trie-cache-size 0 \
    --pool-type=fork-aware \
    $offchain_params \
  ;;

start-paseo-collator-bob)
  printf "\nBuilding frequency with runtime '$parachain' and id '$para_id'...\n"
  cargo build --release --features frequency-local

  parachain_dir_bob=$base_dir/parachain/bob/${para_id}
  mkdir -p $parachain_dir_bob;

  if [ "$2" == "purge" ]; then
    echo "purging parachain..."
    rm -rf $parachain_dir_bob
  fi

  "${Frequency_BINARY_PATH:-./target/release/frequency}" key generate-node-key --base-path=$parachain_dir_bob/data

  ./scripts/run_collator.sh \
    --chain="frequency-paseo-local" --bob \
    --base-path=$parachain_dir_bob/data \
    --wasm-execution=compiled \
    --force-authoring \
    --port $((30332)) \
    --rpc-port $((9943)) \
    --rpc-external \
    --rpc-cors all \
    --rpc-methods=Unsafe \
    --trie-cache-size 0 \
    --pool-type=fork-aware \
    $offchain_params \
  ;;

start-frequency-instant)
  printf "\nBuilding Frequency without relay. Running with instant sealing ...\n"
  cargo build --features frequency-no-relay,force-debug

  parachain_dir=$base_dir/parachain/${para_id}
  mkdir -p $parachain_dir;

  if [ "$2" == "purge" ]; then
    echo "purging parachain..."
    rm -rf $parachain_dir
  fi

  ./target/debug/frequency \
    --dev \
    --state-pruning archive \
    -lbasic-authorship=debug \
    -ltxpool=debug \
    -lruntime=debug \
    --sealing=instant \
    --wasm-execution=compiled \
    --no-telemetry \
    --no-prometheus \
    --port $((30333)) \
    --rpc-port $((9944)) \
    --rpc-external \
    --rpc-cors all \
    --rpc-methods=Unsafe \
    # --pool-type=fork-aware \  Fork aware pool is not supported with insatnt sealing
    $offchain_params \
    --tmp
  ;;

start-frequency-interval)
  defaultInterval=6
  interval=${3-$defaultInterval}
  printf "\nBuilding Frequency without relay.  Running with interval sealing with interval of $interval seconds...\n"
  cargo build --features frequency-no-relay

  parachain_dir=$base_dir/parachain/${para_id}
  mkdir -p $parachain_dir;

  if [ "$2" == "purge" ]; then
    echo "purging parachain..."
    rm -rf $parachain_dir
  fi

  ./target/debug/frequency \
    --dev \
    --state-pruning archive \
    -lbasic-authorship=debug \
    -ltxpool=debug \
    -lruntime=debug \
    --sealing=interval \
    --sealing-interval=${interval} \
    --sealing-create-empty-blocks \   # Fork aware pool is only supported with this feature
    --wasm-execution=compiled \
    --no-telemetry \
    --no-prometheus \
    --port $((30333)) \
    --rpc-port $((9944)) \
    --rpc-external \
    --rpc-cors all \
    --rpc-methods=Unsafe \
    --pool-type=fork-aware \
    $offchain_params \
    --tmp
  ;;

start-frequency-manual)
  printf "\nBuilding frequency without relay.  Running with manual sealing ...\n"
  cargo build --features frequency-no-relay

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
    --sealing=manual \
    --wasm-execution=compiled \
    --no-telemetry \
    --no-prometheus \
    --port $((30333)) \
    --rpc-port $((9944)) \
    --rpc-external \
    --rpc-cors all \
    --rpc-methods=Unsafe \
    --pool-type=fork-aware \
   $offchain_params \
    --tmp
  ;;

start-frequency-container)

  parachain_dir=$base_dir/parachain/${para_id}
  mkdir -p $parachain_dir;
  frequency_default_port=$((30333))
  frequency_default_rpc_port=$((9944))
  frequency_port="${Frequency_PORT:-$frequency_default_port}"
  frequency_rpc_port="${Frequency_RPC_PORT:-$frequency_default_rpc_port}"

  ./scripts/run_collator.sh \
    --chain="frequency-paseo-local" --alice \
    --base-path=$parachain_dir/data \
    --wasm-execution=compiled \
    --force-authoring \
    --port "${frequency_port}" \
    --rpc-port "${frequency_rpc_port}" \
    --rpc-external \
    --rpc-cors all \
    --rpc-methods=Unsafe \
    --trie-cache-size 0 \
    --pool-type=fork-aware \
   $offchain_params \
  ;;

register-frequency-paseo-local)
  echo "reserving and registering parachain with relay via first available slot..."

  cd scripts/js/onboard
  npm i && npm run register "ws://0.0.0.0:9946" "//Alice"
  ;;

onboard-frequency-paseo-local)
  echo "Onboarding parachain with runtime '$parachain' and id '$para_id'..."

   onboard_dir="$base_dir/onboard"
   mkdir -p $onboard_dir
   wasm_location="$onboard_dir/${parachain}-${para_id}.wasm"

   # THE `-r` is important for it to be binary instead of hex
    if [ "$docker_onboard" == "true" ]; then
      genesis=$(docker run -it {REPO_NAME}/frequency:${frequency_docker_image_tag} export-genesis-state --chain="frequency-paseo-local")
      docker run -it {REPO_NAME}/frequency:${frequency_docker_image_tag} export-genesis-wasm --chain="frequency-paseo-local" -r > $wasm_location
    else
      genesis=$(./target/release/frequency export-genesis-state --chain="frequency-paseo-local")
      ./target/release/frequency export-genesis-wasm --chain="frequency-paseo-local" -r > $wasm_location
    fi

  cd scripts/js/onboard
  npm i && npm run onboard "ws://0.0.0.0:9946" "//Alice" ${para_id} "${genesis}" "${wasm_location}"
  ;;

offboard-frequency-paseo-local)
  echo "cleaning up parachain for id '$para_id'..."

  cd scripts/js/onboard
  npm i && npm run cleanup "ws://0.0.0.0:9946" "//Alice" ${para_id}
  ;;

upgrade-frequency-paseo-local)

  root_dir=$(git rev-parse --show-toplevel)
  echo "root_dir is set to $root_dir"

  # Due to defaults and profile=debug, the target directory will be $root_dir/target/debug
  cargo build \
    --package frequency-runtime \
    --features frequency-local

  wasm_location=$root_dir/target/debug/wbuild/frequency-runtime/frequency_runtime.compact.compressed.wasm

  ./scripts/runtime-upgrade.sh "//Alice" "ws://0.0.0.0:9944" $wasm_location

  ./scripts/enact-upgrade.sh "//Alice" "ws://0.0.0.0:9944" $wasm_location

  ;;

upgrade-frequency-no-relay)

  root_dir=$(git rev-parse --show-toplevel)
  echo "root_dir is set to $root_dir"

  # Due to defaults and profile=debug, the target directory will be $root_dir/target/debug
  cargo build \
    --package frequency-runtime \
    --features frequency-no-relay

  wasm_location=$root_dir/target/debug/wbuild/frequency-runtime/frequency_runtime.compact.compressed.wasm

  ./scripts/runtime-dev-upgrade.sh "//Alice" "ws://0.0.0.0:9944" $wasm_location

  ;;

esac
