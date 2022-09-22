.PHONY: start
start:
	./scripts/init.sh start-frequency-instant

start-relay:
	./scripts/init.sh start-relay-chain

start-frequency:
	./scripts/init.sh start-frequency

start-frequency-docker:
	./scripts/init.sh start-frequency-docker

.PHONY: stop
stop-relay:
	./scripts/init.sh stop-relay-chain

stop-frequency-docker:
	./scripts/init.sh stop-frequency-docker

.PHONY: register
register:
	./scripts/init.sh register-frequency

.PHONY: onboard
onboard:
	./scripts/init.sh onboard-frequency

.PHONY: offboard
offboard:
	./scripts/init.sh offboard-frequency

.PHONY: install
install-toolchain:
	./scripts/init.sh install-toolchain

.PHONY: specs
specs-rococo-2000:
	./scripts/generate_specs.sh 2000 rococo-2000 release

specs-rococo-4044:
	./scripts/generate_specs.sh 4044 rococo-4044 release

specs-mainnet:
	./scripts/generate_specs.sh 999 mainnet release

specs-rococo-local:
	./scripts/generate_relay_specs.sh

.PHONY: format
format:
	cargo fmt

.PHONY: lint
lint:
	cargo fmt --check && SKIP_WASM_BUILD=1 env -u RUSTFLAGS cargo clippy --features all-frequency-features -- -D warnings

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

.PHONY: docs
docs:
	./scripts/frequency_docs.sh

# Cleans unused docker resources and artifacts
.PHONY: docs
docker-prune:
	./scripts/prune_all.sh

.PHONY: check
check:
	SKIP_WASM_BUILD= cargo check --features all-frequency-features

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
