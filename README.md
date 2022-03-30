# MRC

## Build

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Initialize your Wasm Build environment:

```bash
./scripts/init.sh install-toolchain
```

Build Wasm and native code:

```bash
cargo build --release
```

Alternatively run

```bash
TARGET=build-node ./ci/build.sh
```

## Run

### Tests

```bash
cargo test
```

Alternatively Run `TARGET=tests ./ci/build.sh` to run cargo tests.

### Configure the environment

```bash
source .env
```

### Start local Relay chain(alice and bob) and Parachain(alice)  

Start relay chain

```bash
./scripts/init.sh start-relay-chain
```

Start mrc as parachain: This step will generate genesis/wasm and onboard the parachain.

Note: assumption is that relay chain is running from step one above and para id 2000 is registered.

```bash
./scripts/init.sh start-parachain
```

Note: set `RUST_LOG=debug RUST_BACKTRACE=1` as the environment variable to enable detailed logs.

Onboarding mrc to Relay chain

```bash
./scripts/init.sh onboard-parachain
```

### Generating a new genesis file

1. Update `node/chain_spec.rs` with required spec config, defaults to `para_id:2000` and relay chain to be `rococo_local.json` with `protocol_id:mrc-local`
2. Run `cargo run --release build-spec --disable-default-bootnode [--chain [name]]> .res/genesis/mrc-spec-rococo.json` to export the chain spec
3. Run `cargo run --release build-spec --disable-default-bootnode .res/genesis/mrc-spec-rococo.json> ./res/genesis/rococo-local-mrc-2000-raw.json` to export the raw chain spec
4. Commit

Note: To build spec against specific chain config; specify chain name in the command above.

### TODO: Refractor chain spec to add genesis config for dev, betanet and polkadot

## Linting

- Lint the project with `cargo +nightly fmt`.
- Linting standards are defined in `rustfmt.toml`.
- Alternatively run `TARGET=lint ./ci/build.sh`

## Verifying Runtime

1. Check out the commit at which the runtime was built.
2. Run `TARGET=build-runtime RUST_TOOLCHAIN=nightly ./ci/build.sh` to use srtool to verify the runtime.
