UNAME := $(shell uname)
NIGHTLY = +nightly-2025-04-03

.PHONY: all
all: build

.PHONY: clean
clean:
	cargo clean

.PHONY: start start-bridging start-bridging-westend start-bridging-westend-local start-frequency start-frequency-docker start-manual start-interval start-interval-short start-with-offchain start-frequency-with-offchain start-manual-with-offchain start-interval-with-offchain
start:
	./scripts/init.sh start-frequency-instant

start-bridging-westend-local:
	./scripts/init.sh start-bridging-westend-local

start-bridging-westend:
	./scripts/init.sh start-bridging-westend
	# TODO: Add testnet support

start-paseo-relay:
	./scripts/init.sh start-paseo-relay-chain


start-paseo-collator-alice:
	./scripts/init.sh start-paseo-collator-alice

start-paseo-collator-bob:
	./scripts/init.sh start-paseo-collator-bob

start-frequency:
	./scripts/init.sh start-frequency

start-frequency-docker:
	./scripts/init.sh start-frequency-docker

run-frequency-docker: start-frequency-docker register onboard-docker

start-relay-chain-docker:
	./scripts/init.sh start-paseo-relay-chain

stop-relay-chain-docker:
	./scripts/init.sh stop-paseo-relay-chain

start-manual:
	./scripts/init.sh start-frequency-manual

start-interval:
	./scripts/init.sh start-frequency-interval

start-interval-short:
	./scripts/init.sh start-frequency-interval 0 1

start-with-offchain:
	./scripts/init.sh start-frequency-instant with-offchain

start-frequency-with-offchain:
	./scripts/init.sh start-frequency with-offchain

start-manual-with-offchain:
	./scripts/init.sh start-frequency-manual with-offchain

start-interval-with-offchain:
	./scripts/init.sh start-frequency-interval with-offchain

.PHONY: stop stop-relay stop-frequency-docker
stop-relay:
	./scripts/init.sh stop-relay-chain

stop-paseo-relay:
	./scripts/init.sh stop-paseo-relay-chain

stop-paseo-relay-prune:
	env PRUNE="--volumes" ./scripts/init.sh stop-paseo-relay-chain

stop-frequency-docker:
	./scripts/init.sh stop-frequency-docker

stop-frequency-docker-prune:
	env PRUNE="--volumes" ./scripts/init.sh stop-frequency-docker

.PHONY: local-block
local-block:
	curl http://localhost:9944 -H "Content-Type:application/json;charset=utf-8" -d   '{ \
		"jsonrpc":"2.0", \
		"id":1, \
		"method":"engine_createBlock", \
		"params": [true, true] \
		}' | jq

.PHONY: register
register:
	./scripts/init.sh register-frequency-paseo-local

.PHONY: onboard
onboard:
	./scripts/init.sh onboard-frequency-paseo-local

.PHONY: onboard-res-local
onboard-res-local:
	./scripts/init.sh onboard-res-local

.PHONY: onboard-docker
onboard-docker:
	env DOCKER_ONBOARD=true PARA_DOCKER_IMAGE=frequencychain/collator-node-local:latest ./scripts/init.sh onboard-frequency-paseo-local

.PHONY: offboard
offboard:
	./scripts/init.sh offboard-frequency-paseo-local

.PHONY: specs-frequency-paseo-local-debug specs-frequency-paseo-local-release

specs-frequency-paseo-local-debug:
	./scripts/generate_specs.sh 2000 frequency-paseo-local debug

specs-frequency-paseo-local-release:
	./scripts/generate_specs.sh 2000 frequency-paseo-local release

specs-frequency-westend-local-release:
	./scripts/generate_specs.sh 2000 frequency-westend-local release

specs-frequency-westend-release:
	./scripts/generate_specs.sh 2313 frequency-westend release

.PHONY: format format-js format-all
format:
	cargo $(NIGHTLY) fmt

format-js:
	cd e2e && npm run format
	cd js/api-augment && npm run format
	cd js/ethereum-utils && npm run format
	cd js/recovery-sdk && npm run format
	cd js/schemas && npm run format

format-all: format format-js

.PHONY: lint lint-audit lint-fix lint-clippy
lint:
	cargo $(NIGHTLY) fmt --check
	SKIP_WASM_BUILD=1 cargo clippy --features runtime-benchmarks,frequency-lint-check -- -Dwarnings
	RUSTC_BOOTSTRAP=1 RUSTDOCFLAGS="--enable-index-page --check -Zunstable-options" cargo doc --no-deps --features frequency

lint-clippy:
	SKIP_WASM_BUILD=1 cargo clippy --features runtime-benchmarks,frequency-lint-check -- -Dwarnings

lint-audit:
	cargo deny check -c deny.toml

lint-fix:
	cargo $(NIGHTLY) fmt
	SKIP_WASM_BUILD=1 cargo clippy --fix --features runtime-benchmarks,frequency-lint-check

.PHONY: format-lint
format-lint: format lint

.PHONY: ci-local
ci-local: check-all lint lint-audit test js e2e-tests

.PHONY: upgrade-local upgrade-no-relay
upgrade-local:
	./scripts/init.sh upgrade-frequency-paseo-local

upgrade-no-relay:
	./scripts/init.sh upgrade-frequency-no-relay


#
# We use hard-coded variables (rather than a pattern) so that smart shells with
# CLI auto-complete for Makefiles will pick up the targets and present as options for auto-complete.
#
# Note: to run benchmarks for > 1 but < all pallets, it is more efficient to call the `run_benchmarks.sh`
#       script directly, as it is able to run the build stage just once for all benchmarks
#
BENCH_TARGETS=\
benchmarks-messages \
benchmarks-msa \
benchmarks-frequency-tx-payment \
benchmarks-overhead \
benchmarks-schemas \
benchmarks-stateful-storage \
benchmarks-handles \
benchmarks-time-release \
benchmarks-passkey \
benchmarks-cumulus_pallet_weight_reclaim \
benchmarks-cumulus_pallet_xcmp_queue \
benchmarks-frame_system_extensions \
benchmarks-pallet_assets \
benchmarks-pallet_balances \
benchmarks-pallet_collator_selection \
benchmarks-pallet_democracy \
benchmarks-pallet_message_queue \
benchmarks-pallet_multisig \
benchmarks-pallet_preimage \
benchmarks-pallet_scheduler \
benchmarks-pallet_session \
benchmarks-pallet_timestamp \
benchmarks-pallet_treasury \
benchmarks-pallet_utility \
benchmarks-pallet_proxy \
benchmarks-pallet_transaction_payment \
benchmarks-pallet_xcm_benchmarks__fungible \
benchmarks-pallet_xcm_benchmarks__generic \

BENCH_LOCAL_TARGETS=\
benchmarks-messages-local \
benchmarks-msa-local \
benchmarks-overhead-local \
benchmarks-schemas-local \
benchmarks-frequency-tx-payment-local \
benchmarks-stateful-storage-local \
benchmarks-handles-local \
benchmarks-passkey-local \
benchmarks-time-release-local \
benchmarks-cumulus_pallet_weight_reclaim-local \
benchmarks-cumulus_pallet_xcmp_queue-local \
benchmarks-frame_system_extensions-local \
benchmarks-pallet_assets-local \
benchmarks-pallet_balances-local \
benchmarks-pallet_collator_selection-local \
benchmarks-pallet_collective-local \
benchmarks-pallet_democracy-local \
benchmarks-pallet_message_queue-local \
benchmarks-pallet_multisig-local \
benchmarks-pallet_preimage-local \
benchmarks-pallet_scheduler-local \
benchmarks-pallet_session-local \
benchmarks-pallet_timestamp-local \
benchmarks-pallet_treasury-local \
benchmarks-pallet_utility-local \
benchmarks-pallet_proxy-local \
benchmarks-pallet_transaction_payment-local \
benchmarks-pallet_xcm_benchmarks__fungible-local \
benchmarks-pallet_xcm_benchmarks__generic-local \

.PHONY: benchmarks
benchmarks:
	./scripts/run_benchmarks.sh

#
# Target to run benchmarks for local development. Uses the "bench-dev" profile,
# since "release" is unnecessary in local development, and by using "bench-dev"
# (which is just a clone of "release"), we don't overwrite our "release" target used
# for development testing.
.PHONY: benchmarks-local
benchmarks-local:
	./scripts/run_benchmarks.sh -t bench-dev

.PHONY: $(BENCH_TARGETS)
$(BENCH_TARGETS):
	./scripts/run_benchmarks.sh $$(echo $(@:benchmarks-%=%) | sed 's/__/::/')

.PHONY: $(BENCH_LOCAL_TARGETS)
$(BENCH_LOCAL_TARGETS):
	./scripts/run_benchmarks.sh -t bench-dev $$(echo $(@:benchmarks-%-local=%) | sed 's/__/::/')

#
# benchmarks-multi-* targets are for ease of use in running benchmarks for multiple
# (but not necessarily all) pallets with a single invocation.
#
.PHONY: benchmarks-multi
benchmarks-multi:
	./scripts/run_benchmarks.sh $(PALLETS)

.PHONY: benchmarks-multi-local
benchmarks-multi-local:
	./scripts/run_benchmarks.sh -t bench-dev $(PALLETS)

.PHONY: docs
docs:
	RUSTC_BOOTSTRAP=1 RUSTDOCFLAGS="--enable-index-page -Zunstable-options" cargo doc --no-deps --workspace --features frequency

# Cleans unused docker resources and artifacts
.PHONY: docs
docker-prune:
	./scripts/prune_all.sh

.PHONY: check-all check check-no-relay check-local check-testnet check-mainnet check-bridging-all check-bridging-mainnet check-bridging-testnet check-bridging-westend check-bridging-local
# Add a target to run all checks to check that all existing features work with the addition of 'frequency-bridging'
# which is an add-on feature and not mutually exclusive with the other features.
check-all: check check-no-relay check-local check-testnet check-mainnet check-bridging-all

check:
	SKIP_WASM_BUILD=1 cargo check --features runtime-benchmarks,frequency-lint-check

check-no-relay:
	SKIP_WASM_BUILD=1 cargo check --features frequency-no-relay

check-local:
	SKIP_WASM_BUILD=1 cargo check --features frequency-local

check-testnet:
	SKIP_WASM_BUILD=1 cargo check --features frequency-testnet

check-mainnet:
	SKIP_WASM_BUILD=1 cargo check --features frequency

check-bridging-all: check-bridging-westend check-bridging-local check-bridging-testnet check-bridging-mainnet

check-bridging-mainnet:
	SKIP_WASM_BUILD=1 cargo check --features frequency,frequency-bridging

check-bridging-testnet:
	SKIP_WASM_BUILD=1 cargo check --features frequency-testnet,frequency-bridging

check-bridging-westend:
	SKIP_WASM_BUILD=1 cargo check --features frequency-westend,frequency-bridging

check-bridging-local:
	SKIP_WASM_BUILD=1 cargo check --features frequency-local,frequency-bridging


.PHONY: js
js:
	./scripts/generate_js_definitions.sh

.PHONY: build build-benchmarks build-no-relay build-local build-testnet build-westend build-mainnet build-testnet-release build-westend-release build-mainnet-release build-bridging-mainnet build-bridging-westend build-bridging-westend-local build-all

build-all: build build-benchmarks build-no-relay build-local build-testnet build-westend build-mainnet build-testnet-release build-westend-release build-mainnet-release build-bridging-mainnet build-bridging-westend build-bridging-westend-local

build:
	cargo build --features frequency-no-relay

build-benchmarks:
	cargo build --release --features runtime-benchmarks,frequency-lint-check --workspace

build-no-relay:
	cargo build --features frequency-no-relay

build-local:
	cargo build --features frequency-local

build-testnet:
	cargo build --features frequency-testnet

build-westend:
	cargo build --features frequency-westend

build-mainnet:
	cargo build --features frequency

build-testnet-release:
	cargo build --locked --features frequency-testnet --release

build-westend-release:
	cargo build --locked --features frequency-westend --release

build-mainnet-release:
	cargo build --locked --features  frequency --release

build-bridging-testnet:
	cargo build --features frequency-testnet,frequency-bridging

build-bridging-mainnet:
	cargo build --features frequency,frequency-bridging

build-bridging-westend:
	cargo build --features frequency-westend,frequency-bridging --release

build-bridging-local:
	cargo build --features frequency-local,frequency-bridging --release

.PHONY: test test-bridging e2e-tests e2e-tests-serial e2e-tests-only e2e-tests-load e2e-tests-load-only e2e-tests-testnet-paseo e2e-tests-paseo-local
test:
	cargo test --workspace --features runtime-benchmarks,frequency-lint-check

# Convenience targets to run individual package/pallet tests
# To further limit to specific tests, specify `make TEST=<test-name-pattern> <test-target>`
test-frequency-runtime \
test-common-runtime \
test-common-primitives \
test-common-helpers \
test-pallet-capacity \
test-pallet-frequency-tx-payment \
test-pallet-handles \
test-pallet-messages \
test-pallet-msa \
test-pallet-passkey \
test-pallet-schemas \
test-pallet-stateful-storage \
test-pallet-time-release \
test-pallet-treasury@27.0.0 \
test-cli-opt:
	cargo test -p $(@:test-%=%) --lib -- $(TEST)

test-migrations:
	cargo test --workspace --features runtime-benchmarks,frequency-lint-check,try-runtime

e2e-tests:
	./scripts/run_e2e_tests.sh

e2e-tests-serial:
	./scripts/run_e2e_tests.sh -c serial

e2e-tests-only:
	./scripts/run_e2e_tests.sh -s

e2e-tests-load:
	./scripts/run_e2e_tests.sh load

e2e-tests-load-only:
	./scripts/run_e2e_tests.sh -s load

e2e-tests-testnet-paseo:
	./scripts/run_e2e_tests.sh -c paseo_testnet

e2e-tests-paseo-local:
	./scripts/run_e2e_tests.sh -c paseo_local

.PHONY: check-try-runtime-installed
check-try-runtime-installed:
	@which try-runtime > /dev/null || (echo "try-runtime is not installed. Please install it" && exit 1)

MAINNET_URI=wss://1.rpc.frequency.xyz:443
PASEO_URI=wss://0.rpc.testnet.amplica.io:443
WESTEND_URI=wss://node-7330371704012918784.nv.onfinality.io/ws?apikey=$(ONFINALITY_APIKEY)
LOCAL_URI=ws://localhost:9944
WASM_PATH=./target/release/wbuild/frequency-runtime/frequency_runtime.wasm
# Without the state from this minimal set of pallets, try-runtime panics when trying to validate multi-block migrations
MINIMAL_PALLETS=ParachainSystem ParachainInfo System Timestamp Aura Authorship
TRY_RUNTIME_BUILD_TYPE=release
# Result of `echo -n ":child_storage:default:" | xxd -p -c 256`
DEFAULT_CHILD_TREE_PREFIX=0x3a6368696c645f73746f726167653a64656661756c743a

.PHONY: check-onfinality-api-key
check-onfinality-api-key:
	@if [ "$(CHAIN)" == "testnet-westend" -a -z "$(ONFINALITY_APIKEY)" ]; then \
		echo "Error: ONFINALITY_APIKEY environment variable is not set. Please set it before running this target."; \
		echo "Example: ONFINALITY_APIKEY=your-api-key-here make try-runtime-check-migrations-westend-testnet"; \
		exit 1; \
	fi

space := $(subst ,, )

try-runtime-%: PALLET_FLAGS=$(if $(strip $(PALLETS)),$(PALLETS:%=-p %) $(MINIMAL_PALLETS:%=-p %),)
try-runtime-%: SNAPSHOT_PALLETS=$(if $(strip $(PALLETS)),$(subst  $(space),-,$(strip $(PALLETS))),all-pallets)
try-runtime-%: CHILD_TREE_FLAGS=$(if $(filter true,$(CHILD_TREES)), --child-tree --prefix $(DEFAULT_CHILD_TREE_PREFIX),)
try-runtime-%-paseo-testnet try-runtime-%-bridging-testnet: URI := $(PASEO_URI)
try-runtime-%-paseo-testnet try-runtime-%-bridging-testnet: CHAIN := testnet-paseo
try-runtime-%-westend-testnet: URI := $(WESTEND_URI)
try-runtime-%-westend-testnet: CHAIN := testnet-westend
try-runtime-%-mainnet: URI := $(MAINNET_URI)
try-runtime-%-mainnet: CHAIN := mainnet
try-runtime-%-local: URI := $(LOCAL_URI)
try-runtime-%-local: CHAIN := local
try-runtime-%-local: WASM_PATH=./target/debug/wbuild/frequency-runtime/frequency_runtime.wasm


build-runtime-paseo-testnet: FEATURES := frequency-testnet
build-runtime-bridging-testnet: FEATURES := frequency-testnet,frequency-bridging
build-runtime-mainnet: FEATURES := frequency
build-runtime-westend-testnet: FEATURES := frequency-westend,frequency-bridging
build-runtime-local: FEATURES := frequency-no-relay
build-runtime-local: TRY_RUNTIME_BUILD_TYPE := dev

.PHONY: build-runtime-paseo-testnet build-runtime-westend-testnet build-runtime-mainnet build-runtime-local
build-runtime-local \
build-runtime-paseo-testnet \
build-runtime-westend-testnet \
build-runtime-mainnet:
	cargo build --package frequency-runtime --profile ${TRY_RUNTIME_BUILD_TYPE} --features $(FEATURES),try-runtime --locked

#
# The 'try-runtime' targets can optionally be constrained to fetch state for only specific pallets. This is useful to
# avoid unnecessarily fetching large state trees for pallets not under test. The list of pallets is:
# Msa Messages StatefulStorage Capacity FrequencyTxPayment Handles Passkey Schemas

.PHONY: try-runtime-create-snapshot-paseo-testnet try-runtime-create-snapshot-westend-testnet try-runtime-create-snapshot-mainnet try-runtime-create-snapshot-local
try-runtime-create-snapshot-local \
try-runtime-create-snapshot-paseo-testnet \
try-runtime-create-snapshot-westend-testnet \
try-runtime-create-snapshot-mainnet: check-try-runtime-installed check-onfinality-api-key
	try-runtime create-snapshot $(PALLET_FLAGS) $(CHILD_TREE_FLAGS) --uri $(URI) $(CHAIN)-$(SNAPSHOT_PALLETS).state


.PHONY: try-runtime-upgrade-paseo-testnet try-runtime-upgrade-mainnet
try-runtime-upgrade-paseo-testnet \
try-runtime-upgrade-mainnet: try-runtime-upgrade-%: check-try-runtime-installed build-runtime-%
	try-runtime --runtime $(WASM_PATH) on-runtime-upgrade --blocktime=6000 live $(PALLET_FLAGS) $(CHILD_TREE_FLAGS) --uri $(URI)

.PHONY: try-runtime-use-snapshot-paseo-testnet try-runtime-use-snapshot-mainnet try-runtime-use-snapshot-local
try-runtime-use-snapshot-local \
try-runtime-use-snapshot-paseo-testnet \
try-runtime-use-snapshot-mainnet: try-runtime-use-snapshot-%: check-try-runtime-installed build-runtime-%
	try-runtime --runtime $(WASM_PATH) on-runtime-upgrade --blocktime=6000 snap --path $(CHAIN)-$(SNAPSHOT_PALLETS).state

.PHONY: try-runtime-check-migrations-paseo-testnet try-runtime-check-migrations-bridging-testnet try-runtime-check-migrations-westend-testnet
try-runtime-check-migrations-paseo-testnet \
try-runtime-check-migrations-bridging-testnet \
try-runtime-check-migrations-westend-testnet: try-runtime-check-migrations-%: check-try-runtime-installed check-onfinality-api-key build-runtime-%
	try-runtime --runtime $(WASM_PATH) on-runtime-upgrade --blocktime=6000 --checks="pre-and-post" --disable-spec-version-check live --uri $(URI) $(PALLET_FLAGS) $(CHILD_TREE_FLAGS)

.PHONY: try-runtime-check-migrations-local
try-runtime-check-migrations-local: check-try-runtime-installed build-runtime-local
	try-runtime --runtime $(WASM_PATH) on-runtime-upgrade --blocktime=6000 --checks="pre-and-post" --disable-spec-version-check live --uri $(URI) $(PALLET_FLAGS) $(CHILD_TREE_FLAGS)

.PHONY: try-runtime-check-migrations-none-local
try-runtime-check-migrations-none-local: check-try-runtime-installed build-runtime-local
	try-runtime --runtime $(WASM_PATH) on-runtime-upgrade --blocktime=6000 --checks="none" --disable-spec-version-check live --uri $(URI) $(PALLET_FLAGS) $(CHILD_TREE_FLAGS)

# Pull the Polkadot version from the polkadot-cli package in the Cargo.lock file.
# This will break if the lock file format changes
POLKADOT_VERSION=$(shell grep "^polkadot-cli" Cargo.toml | grep -o 'tag[[:space:]]*=[[:space:]]*"\(.*\)"' | sed 's/tag *= *"polkadot-\(.*\)"/\1/' | head -n 1)

.PHONY: version
version:
ifndef v
	@echo "Please set the version with v=#.#.#[-#]"
	@exit 1
endif
ifneq (,$(findstring v,  $(v)))
	@echo "Please don't prefix with a 'v'. Use: v=#.#.#[-#]"
	@exit 1
endif
ifeq (,$(POLKADOT_VERSION))
	@echo "Error: Having trouble finding the Polkadot version. Sorry about that.\nCheck my POLKADOT_VERSION variable command."
	@exit 1
endif
	@echo "Setting the crate versions to "$(v)+polkadot$(POLKADOT_VERSION)
ifeq ($(UNAME), Linux)
	$(eval $@_SED := -i -e)
endif
ifeq ($(UNAME), Darwin)
	$(eval $@_SED := -i '')
endif
	find . -type f -name "Cargo.toml" -print0 | xargs -0 sed $($@_SED) 's/^version = \"0\.0\.0\"/version = \"$(v)+polkadot$(POLKADOT_VERSION)\"/g';
	@echo "Doing cargo check for just examples seems to be the easiest way to update version in Cargo.lock"
	cargo check --examples --quiet
	@echo "All done. Don't forget to double check that the automated replacement worked."

.PHONY: version-polkadot
version-polkadot:
ifeq (,$(POLKADOT_VERSION))
	@echo "Error: Having trouble finding the Polkadot version. Sorry about that.\nCheck my POLKADOT_VERSION variable command."
	@exit 1
endif
	@echo $(POLKADOT_VERSION)

.PHONY: version-reset
version-reset:
	find ./ -type f -name 'Cargo.toml' -exec sed -i '' 's/^version = \".*+polkadot.*\"/version = \"0.0.0\"/g' {} \;

.PHONY: genesis-schemas
genesis-schemas: js
	cd tools/genesis-data && \
	npm i && \
	npm run --silent schemas > ../../resources/genesis-schemas.json
