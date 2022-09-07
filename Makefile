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
	./scripts/generate_specs.sh 2000 rococo-2000

specs-rococo-4044:
	./scripts/generate_specs.sh 4044 rococo-4044

specs-mainnet:
	./scripts/generate_specs.sh 999 mainnet

.PHONY: format
format:
	cargo fmt -- --check

.PHONY: lint
lint:
	SKIP_WASM_BUILD=1 env -u RUSTFLAGS cargo clippy --all-targets

.PHONY: format-lint
format-lint: format lint

.PHONY: upgrade
upgrade-local:
	./scripts/init.sh upgrade-frequency


.PHONY: benchmarks
benchmarks:
	./scripts/run_all_benchmarks.sh

.PHONY: docs
docs:
	./scripts/frequency_docs.sh

# Cleans unused docker resources and artifacts
.PHONY: docs
docker-prune:
	./scripts/prune_all.sh
