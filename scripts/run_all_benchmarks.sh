#!/usr/bin/env bash

export RUST_LOG=info
THIS_DIR=$( dirname -- "$0"; )
PROJECT="${THIS_DIR}/.."
PROFILE=production
ALL_EXTERNAL_PALLETS=( \
  orml_vesting \
  pallet_balances \
  pallet_collator_selection \
  pallet_collective \
  pallet_democracy \
  pallet_multisig \
  pallet_preimage \
  pallet_scheduler \
  pallet_session \
  pallet_timestamp \
  pallet_treasury \
  pallet_utility \
)
ALL_CUSTOM_PALLETS=( \
  messages \
  msa \
  schemas \
  stateful-storage \
)

declare -a CUSTOM_PALLETS
declare -a EXTERNAL_PALLETS
skip_build=false
OVERHEAD=
PALLET=

function exit_err() { echo "❌ 💔" ; exit 1; }

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

while getopts 'd:p:st:v' flag; do
  case "${flag}" in
    d)
      # Set project directory
      PROJECT="${OPTARG}"
      ;;
    p)
      # Single pallet run
      PALLET="${OPTARG}"
      ;;
    s)
      # Skip build.
      skip_build=true
      ;;
    t)
      # Set target profile
      case ${OPTARG} in
        production|release|debug)
          PROFILE="${OPTARG}"
          ;;
        *) echo "Unrecognized target profile: ${OPTARG}"
           exit_err
           ;;
      esac
      ;;
    v)
      # Echo all executed commands.
      set -x
      ;;
    ?)
      # Unrecognized option. getopts will log the error
      exit 1
      ;;
    *)
      # Exit early.
      echo "Bad options. Check Script."
      exit 1
      ;;
  esac
done
shift $(( ${OPTIND} - 1 ))

for pallet in "${@}"
do
   if [[ -n "${PALLET}" ]]
   then
      echo "-p <pallet> and pallet args are mutually exclusive."
      exit_err
   elif is_external_pallet ${pallet}
   then
      EXTERNAL_PALLETS=( "${EXTERNAL_PALLETS[@]}" "${pallet}" )
   elif is_custom_pallet ${pallet}
   then
      CUSTOM_PALLETS=( "${CUSTOM_PALLETS[@]}" "pallet_${pallet}" )
   elif [[ "${pallet}" == "overhead" ]]
   then
      OVERHEAD=overhead
   else
      echo "Unrecognized pallet: ${pallet}"
      exit_err
   fi
done

if [[ -n "${PALLET}" ]]
then
   if is_external_pallet ${pallet}
   then
      EXTERNAL_PALLETS=( "${pallet}" )
   elif is_custom_pallet ${pallet}
   then
      CUSTOM_PALLETS=( "pallet_${pallet}" )
   elif [[ "${PALLET}" == "overhead" ]]
   then
      OVERHEAD=overhead
   else
      echo "Unrecognized pallet: ${PALLET}"
      exit_err
   fi
fi

if [[ ${#EXTERNAL_PALLETS[@]} == 0 && ${#CUSTOM_PALLETS[@]} == 0 && -z "${OVERHEAD}" ]]
then
  EXTERNAL_PALLETS=( "${ALL_EXTERNAL_PALLETS[@]}" )
  CUSTOM_PALLETS=( ${ALL_CUSTOM_PALLETS[@]/#/pallet_} )
  OVERHEAD=overhead
fi

RUNTIME=${PROJECT}/target/${PROFILE}/frequency
BENCHMARK="${RUNTIME} benchmark "

echo "Running benchmarks for the following pallets:\
${EXTERNAL_PALLETS[@]} \
${CUSTOM_PALLETS[@]} \
${OVERHEAD}"


function run_benchmark() {
  echo "Running benchmarks for ${1}"
  echo " "
  ${BENCHMARK} pallet \
  --pallet ${1} \
  --extrinsic "*" \
  --chain="frequency-bench" \
  --execution wasm \
  --heap-pages=4096 \
  --wasm-execution compiled \
  --steps=${2} \
  --repeat=${3} \
  --output=${4} \
  --template=${5}
}

if [[ ${skip_build} == false ]]
then
  cargo build --profile=${PROFILE} --features runtime-benchmarks --features all-frequency-features --workspace || exit_err
fi

for external_pallet in "${EXTERNAL_PALLETS[@]}"; do
  output=${PROJECT}/runtime/common/src/weights/${external_pallet}.rs
  steps=50
  repeat=20
  template=${PROJECT}/.maintain/runtime-weight-template.hbs
  run_benchmark ${external_pallet} ${steps} ${repeat} ${output} ${template} || exit_err
done

for pallet_name in "${CUSTOM_PALLETS[@]}"; do
  steps=20
  repeat=10
  template=${PROJECT}/.maintain/frame-weight-template.hbs
  output=${PROJECT}/pallets/${pallet_name}/src/weights.rs
  run_benchmark ${pallet_name} ${steps} ${repeat} ${output} ${template} || exit_err
done

if [[ -n "${OVERHEAD}" ]]
then
  echo "Running extrinsic and block overhead benchmark"
  echo " "
  ${BENCHMARK} overhead --execution=wasm --wasm-execution=compiled --weight-path=runtime/common/src/weights --chain=dev --warmup=10 --repeat=100 || exit_err
fi
