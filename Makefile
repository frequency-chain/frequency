UNAME := $(shell uname)

.PHONY: all
all: build

.PHONY: clean
clean:
	cargo clean

.PHONY: start
start:
	./scripts/init.sh start-frequency-instant

start-relay:
	./scripts/init.sh start-relay-chain

start-frequency:
	./scripts/init.sh start-frequency

start-frequency-docker:
	./scripts/init.sh start-frequency-docker

start-manual:
	./scripts/init.sh start-frequency-manual

.PHONY: stop
stop-relay:
	./scripts/init.sh stop-relay-chain

stop-frequency-docker:
	./scripts/init.sh stop-frequency-docker

.PHONY: local-block
local-block:
	curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d   '{ \
		"jsonrpc":"2.0", \
		"id":1, \
		"method":"engine_createBlock", \
		"params": [true, true] \
		}' | jq

.PHONY: register
register:
	./scripts/init.sh register-frequency-rococo-local

.PHONY: onboard
onboard:
	./scripts/init.sh onboard-frequency-rococo-local

.PHONY: offboard
offboard:
	./scripts/init.sh offboard-frequency-rococo-local

.PHONY: specs
specs-rococo-2000:
	./scripts/generate_specs.sh 2000 rococo-2000 release

specs-rococo-local:
	./scripts/generate_relay_specs.sh

.PHONY: format
format:
	cargo fmt

.PHONY: lint
lint:
	cargo fmt --check
	SKIP_WASM_BUILD=1 env -u RUSTFLAGS cargo clippy --features runtime-benchmarks,frequency-lint-check -- -D warnings
	RUSTDOCFLAGS="--enable-index-page --check -Zunstable-options" cargo doc --no-deps --features frequency

lint-audit:
	cargo deny check -c .cargo-deny.toml

.PHONY: format-lint
format-lint: format lint

.PHONY: ci-local
ci-local: check lint lint-audit test integration-test

.PHONY: upgrade
upgrade-local:
	./scripts/init.sh upgrade-frequency-rococo-local

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
benchmarks-handles\
benchmarks-time-release\
benchmarks-pallet_balances \
benchmarks-pallet_collator_selection \
benchmarks-pallet_democracy \
benchmarks-pallet_multisig \
benchmarks-pallet_preimage \
benchmarks-pallet_scheduler \
benchmarks-pallet_session \
benchmarks-pallet_timestamp \
benchmarks-pallet_treasury \
benchmarks-pallet_utility

BENCH_LOCAL_TARGETS=\
benchmarks-messages-local \
benchmarks-msa-local \
benchmarks-overhead-local \
benchmarks-schemas-local \
benchmarks-frequency-tx-payment-local \
benchmarks-stateful-storage-local \
benchmarks-handles-local \
benchmarks-time-release-local \
benchmarks-pallet_balances-local \
benchmarks-pallet_collator_selection-local \
benchmarks-pallet_collective-local \
benchmarks-pallet_democracy-local \
benchmarks-pallet_multisig-local \
benchmarks-pallet_preimage-local \
benchmarks-pallet_scheduler-local \
benchmarks-pallet_session-local \
benchmarks-pallet_timestamp-local \
benchmarks-pallet_treasury-local \
benchmarks-pallet_utility-local

.PHONY: benchmarks
benchmarks:
	./scripts/run_benchmarks.sh

#
# Target to run benchmarks for local development. Uses the "bench-dev" profile,
# since "production" is unnecessary in local development, and by using "bench-dev"
# (which is just a clone of "release"), we don't overwrite our "release" target used
# for development testing.
.PHONY: benchmarks-local
benchmarks-local:
	./scripts/run_benchmarks.sh -t bench-dev

.PHONY: $(BENCH_TARGETS)
$(BENCH_TARGETS):
	./scripts/run_benchmarks.sh $(@:benchmarks-%=%)

.PHONY: $(BENCH_LOCAL_TARGETS)
$(BENCH_LOCAL_TARGETS):
	./scripts/run_benchmarks.sh -t bench-dev $(@:benchmarks-%-local=%)

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

benchmarks-capacity:
	./scripts/run_benchmark.sh -p capacity

.PHONY: docs
docs:
	RUSTDOCFLAGS="--enable-index-page -Zunstable-options" cargo doc --no-deps --features frequency

# Cleans unused docker resources and artifacts
.PHONY: docs
docker-prune:
	./scripts/prune_all.sh

.PHONY: check
check:
	SKIP_WASM_BUILD= cargo check --features runtime-benchmarks,frequency-lint-check

check-no-relay:
	SKIP_WASM_BUILD= cargo check --features frequency-no-relay

check-local:
	SKIP_WASM_BUILD= cargo check --features frequency-rococo-local

check-rococo:
	SKIP_WASM_BUILD= cargo check --features frequency-rococo-testnet

check-mainnet:
	SKIP_WASM_BUILD= cargo check --features frequency

.PHONY: js
js:
	./scripts/generate_js_definitions.sh

.PHONY: build
build:
	cargo build --locked --release --features frequency-no-relay

build-benchmarks:
	cargo build --profile production --features runtime-benchmarks,frequency-lint-check --workspace

build-no-relay:
	cargo build --locked --features frequency-no-relay

build-local:
	cargo build --locked --features frequency-rococo-local

build-rococo:
	cargo build --locked --release --features frequency-rococo-testnet

build-mainnet:
	cargo build --locked --release --features frequency

build-rococo-release:
	cargo build --locked --features frequency-rococo-testnet --profile production

build-mainnet-release:
	cargo build --locked --features  frequency --profile production

.PHONY: test
test:
	cargo test --workspace --locked --features runtime-benchmarks,frequency-lint-check

integration-test:
	./scripts/run_integration_tests.sh

integration-test-only:
	./scripts/run_integration_tests.sh -s

integration-test-load:
	./scripts/run_integration_tests.sh load

integration-test-load-only:
	./scripts/run_integration_tests.sh -s load

.PHONY: try-runtime
try-runtime:
	cargo run --release --features frequency-lint-check,try-runtime try-runtime --help

try-runtime-upgrade-rococo:
	cargo build --release --features frequency-rococo-testnet,try-runtime
	cargo run --release --features frequency-lint-check,try-runtime try-runtime --runtime ./target/release/wbuild/frequency-runtime/frequency_runtime.wasm on-runtime-upgrade --checks live --uri wss://rpc.rococo.frequency.xyz:443

try-runtime-upgrade-mainnet:
	cargo build --release --features frequency,try-runtime
	cargo run --release --features frequency-lint-check,try-runtime try-runtime --runtime ./target/release/wbuild/frequency-runtime/frequency_runtime.wasm on-runtime-upgrade --checks live --uri wss://1.rpc.frequency.xyz:443

# Pull the Polkadot version from the polkadot-cli package in the Cargo.lock file.
# This will break if the lock file format changes
POLKADOT_VERSION=$(shell awk -F "=" '/name = "polkadot-cli"/,/version = ".*"/{ print $2 }' Cargo.lock | tail -n 1 | cut -d " " -f 3 | tr -d \")

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
