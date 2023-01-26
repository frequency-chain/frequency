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
	./scripts/init.sh register-frequency

.PHONY: onboard
onboard:
	./scripts/init.sh onboard-frequency

.PHONY: offboard
offboard:
	./scripts/init.sh offboard-frequency

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
	SKIP_WASM_BUILD=1 env -u RUSTFLAGS cargo clippy --features runtime-benchmarks,all-frequency-features -- -D warnings
	RUSTDOCFLAGS="--enable-index-page --check -Zunstable-options" cargo doc --no-deps --features frequency

lint-audit:
	cargo deny check -c .cargo-deny.toml

.PHONY: format-lint
format-lint: format lint

.PHONY: upgrade
upgrade-local:
	./scripts/init.sh upgrade-frequency


.PHONY: benchmarks
benchmarks:
	./scripts/run_all_benchmarks.sh

benchmarks-msa:
	./scripts/run_benchmark.sh -p msa

benchmarks-messages:
	./scripts/run_benchmark.sh -p messages

benchmarks-schemas:
	./scripts/run_benchmark.sh -p schemas

benchmarks-stateful-storage:
	./scripts/run_benchmark.sh -p stateful-storage

.PHONY: docs
docs:
	RUSTDOCFLAGS="--enable-index-page -Zunstable-options" cargo doc --no-deps --features frequency

# Cleans unused docker resources and artifacts
.PHONY: docs
docker-prune:
	./scripts/prune_all.sh

.PHONY: check
check:
	SKIP_WASM_BUILD= cargo check --features runtime-benchmarks,all-frequency-features

check-local:
	SKIP_WASM_BUILD= cargo check --features  frequency-rococo-local

check-rococo:
	SKIP_WASM_BUILD= cargo check --features  frequency-rococo-testnet

check-mainnet:
	SKIP_WASM_BUILD= cargo check --features  frequency

.PHONY: js
js:
	./scripts/generate_js_definitions.sh

.PHONY: build
build:
	cargo build --locked --release --features all-frequency-features

build-benchmarks:
	cargo build --profile production --features runtime-benchmarks --features all-frequency-features --workspace

build-local:
	cargo build --locked --release --features  frequency-rococo-local

build-rococo:
	cargo build --locked --release --features  frequency-rococo-testnet

build-mainnet:
	cargo build --locked --release --features  frequency

build-rococo-release:
	cargo build --locked --features  frequency-rococo-testnet --profile production

build-mainnet-release:
	cargo build --locked --features  frequency --profile production

.PHONY: test
test:
	cargo test --workspace --locked --features runtime-benchmarks,all-frequency-features

integration-test:
	./scripts/run_integration_tests.sh

# Pull the Polkadot version from the polkadot-cli package in the Cargo.lock file.
# This will break if the lock file format changes
POLKADOT_VERSION=$(shell awk -F "=" '/name = "polkadot-cli"/,/version = ".*"/{ print $2 }' Cargo.lock | tail -n 1 | cut -d " " -f 3 | tr -d \")

.PHONY: version
version:
ifndef v
	@echo "Please set the version with v=X.X.X-X"
	@exit 1
endif
ifneq (,$(findstring v,  $(v)))
	@echo "Please don't prefix with a 'v'. Use: v=X.X.X-X"
	@exit 1
endif
ifeq (,$(POLKADOT_VERSION))
	@echo "Error: Having trouble finding the Polkadot version. Sorry about that.\nCheck my POLKADOT_VERSION variable command."
	@exit 1
endif
	@echo "Setting the crate versions to "$(v)+polkadot$(POLKADOT_VERSION)
	find ./ -type f -name 'Cargo.toml' -exec sed -i '' 's/^version = \"0\.0\.0\"/version = \"$(v)+polkadot$(POLKADOT_VERSION)\"/g' {} \;
	cargo check
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
