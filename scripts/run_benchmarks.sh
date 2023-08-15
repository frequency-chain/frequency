#!/usr/bin/env bash

export RUST_LOG=info
THIS_DIR=$( dirname -- "$0"; )
PROJECT="${THIS_DIR}/.."
PROFILE=release
PROFILE_DIR=${PROFILE}

ALL_EXTERNAL_PALLETS=( \
  pallet_balances \
  pallet_collator_selection \
  pallet_collective \
  pallet_democracy \
  pallet_multisig \
  pallet_preimage \
  pallet_scheduler \
  pallet_session \
  pallet_sudo \
  pallet_timestamp \
  pallet_treasury \
  pallet_utility \
)
ALL_CUSTOM_PALLETS=( \
  messages \
  msa \
  schemas \
  stateful-storage \
  handles \
  time-release \
  capacity \
  frequency-tx-payment \
)

declare -a CUSTOM_PALLETS
declare -a EXTERNAL_PALLETS
skip_build=false
OVERHEAD=
VERBOSE=

function exit_err() { echo "❌ 💔" ; exit 1; }

function usage() {
  cat << EOI
  Usage: $( basename ${1} ) [-d <dir>] [-p <pallet] [-s] [-t <profile>] [-v]
         $( basename ${1} ) [-d <dir>] [-s] [-t] [-v] [<pallet1> [... <palletN>]]

         -d <dir>     Sets top-level repository directory to <dir>.
                      Default: parent directory of script

        -h            Display this message and exit.

        -s            Skip the build step; use existing binary for the current profile

        -t <profile>  Use '--profile=<profile>' in the build step & for locating the
                      resulting binary. Valid targets are: dev,release,bench-dev

                      (NOTE: using the 'bench-dev' profile will generate a warning in the WASM build.
                       this can safely be ignored. 'bench-dev' is a clone of 'release' and is useful
                       for running benchmarks locally while not overwriting any existing 'release' target.)

        -v            Verbose mode. All shell commands are echoed.

        <palletX>     To run for multiple specific pallets. If no pallets
                      specified, will run all benchmarks.

EOI
}

function is_external_pallet() {
  for p in "${ALL_EXTERNAL_PALLETS[@]}"
  do
     if [ "${1}" == "${p}" ]
     then
        return 0
     fi
  done

  return 1
}

function is_custom_pallet() {
  for p in "${ALL_CUSTOM_PALLETS[@]}"
  do
     if [ "${1}" == "${p}" ]
     then
        return 0
     fi
  done

  return 1
}

while getopts 'dh:p:st:v' flag; do
  case "${flag}" in
    d)
      # Set project directory
      PROJECT="${OPTARG}"
      ;;
    h)
      usage ${0}
      exit 0
      ;;
    s)
      # Skip build.
      skip_build=true
      ;;
    t)
      # Set target profile
      case ${OPTARG} in
        release|dev|bench-dev)
          PROFILE="${OPTARG}"
          PROFILE_DIR=${PROFILE}
          # For historical reasons, cargo puts dev builds in the "debug" directory
          if [ ${PROFILE} = "dev" ]
          then
            PROFILE_DIR=debug
          fi
          ;;
        *) echo "Unrecognized target profile: ${OPTARG}"
           usage ${0}
           exit_err
           ;;
      esac
      ;;
    v)
      # Echo all executed commands.
      VERBOSE=1
      set -x
      ;;
    ?)
      # Unrecognized option. getopts will log the error
      usage ${0}
      exit_err
      ;;
    *)
      # Exit early.
      echo "Bad options. Check Script."
      usage ${0}
      exit_err
      ;;
  esac
done
shift $(( ${OPTIND} - 1 ))

for pallet in "${@}"
do
   if is_external_pallet ${pallet}
   then
      EXTERNAL_PALLETS=( "${EXTERNAL_PALLETS[@]}" "${pallet}" )
   elif is_custom_pallet ${pallet}
   then
      CUSTOM_PALLETS=( "${CUSTOM_PALLETS[@]}" "${pallet}" )
   elif [[ "${pallet}" == "overhead" ]]
   then
      OVERHEAD=overhead
   else
      echo "Unrecognized pallet: ${pallet}"
      exit_err
   fi
done

if [[ ${#EXTERNAL_PALLETS[@]} == 0 && ${#CUSTOM_PALLETS[@]} == 0 && -z "${OVERHEAD}" ]]
then
  EXTERNAL_PALLETS=( "${ALL_EXTERNAL_PALLETS[@]}" )
  CUSTOM_PALLETS=( ${ALL_CUSTOM_PALLETS[@]} )
  OVERHEAD=overhead
fi

RUNTIME=${PROJECT}/target/${PROFILE_DIR}/frequency
BENCHMARK="${RUNTIME} benchmark "

echo "Running benchmarks for the following pallets:\
${EXTERNAL_PALLETS[@]} \
${CUSTOM_PALLETS[@]} \
${OVERHEAD}"


function run_benchmark() {
  echo "Running benchmarks for ${1}"
  set -x
  ${BENCHMARK} pallet \
  --pallet=${1} \
  --extrinsic "*" \
  --chain="frequency-bench" \
  --execution=wasm \
  --heap-pages=4096 \
  --wasm-execution=compiled \
  --additional-trie-layers=20 \
  --steps=${2} \
  --repeat=${3} \
  --output=${4} \
  --template=${5}
  if [ -z "${VERBOSE}" ]
  then
    set +x
  fi
}

if [[ ${skip_build} == false ]]
then
  CMD="cargo build --profile=${PROFILE} --features=runtime-benchmarks,frequency-lint-check --workspace"
  echo ${CMD}
  ${CMD} || exit_err
fi

for external_pallet in "${EXTERNAL_PALLETS[@]}"; do
  output=${PROJECT}/runtime/common/src/weights
  steps=50
  repeat=20
  template=${PROJECT}/.maintain/runtime-weight-template.hbs
  run_benchmark ${external_pallet} ${steps} ${repeat} ${output} ${template} || exit_err
done

for pallet_name in "${CUSTOM_PALLETS[@]}"; do
  steps=20
  repeat=10
  template=${PROJECT}/.maintain/frame-weight-template.hbs
  output=${PROJECT}/pallets/${pallet_name/_/-}/src/weights.rs
  run_benchmark pallet_${pallet_name} ${steps} ${repeat} ${output} ${template} || exit_err
done

if [[ -n "${OVERHEAD}" ]]
then
  echo "Running extrinsic and block overhead benchmark"
  echo " "
  ${BENCHMARK} overhead --execution=wasm --wasm-execution=compiled --weight-path=runtime/common/src/weights --chain=dev --warmup=10 --repeat=100 --header="./HEADER-APACHE2" || exit_err
fi