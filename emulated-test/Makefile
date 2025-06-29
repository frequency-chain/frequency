UNAME := $(shell uname)
NIGHTLY = +nightly-2025-04-03

.PHONY: all
all: build

.PHONY: clean
clean:
	cargo clean

.PHONY: format
format:
	cargo $(NIGHTLY) fmt

.PHONY: lint lint-audit lint-fix lint-clippy
lint:
	cargo $(NIGHTLY) fmt --check
	SKIP_WASM_BUILD=1 cargo clippy -- -Dwarnings
	RUSTC_BOOTSTRAP=1 RUSTDOCFLAGS="--enable-index-page --check -Zunstable-options" cargo doc --no-deps

lint-clippy:
	SKIP_WASM_BUILD=1 cargo clippy -- -Dwarnings

lint-audit:
	cargo deny check -c deny.toml

lint-fix:
	cargo $(NIGHTLY) fmt
	SKIP_WASM_BUILD=1 cargo clippy --fix

.PHONY: format-lint
format-lint: format lint

check:
	SKIP_WASM_BUILD= cargo check
