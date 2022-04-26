# MRC

## Build

Install Docker and Docker Compose:

1. Follow the instructions on [Docker](https://docs.docker.com/engine/install/)
2. Follow instructions on [Docker Compose](https://docs.docker.com/compose/install/)

Note: For mac users, [Docker Desktop](https://docs.docker.com/desktop/mac/install/) also installs docker compose environment.

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

### Start local Relay chain(alice and bob) and MRC(alice)  

1. Start relay chain

    ```bash
    ./scripts/init.sh start-relay-chain
    ```

1. Relay chain is running on port `9945` and `9946` for alice and bob respectively.

1. Start mrc as parachain: This step will generate genesis/wasm and onboard the parachain. If new pallets or runtime code changes have been made to mrc, then developer have to generate chain specs again. Refer to [generation spec file](#generating-a-new-spec-file) for more details.

1. Note: assumption is that relay chain is running from step one above and para id 2000 is registered on relay.

    ```bash
    ./scripts/init.sh start-mrc
    ```

1. Note: set `RUST_LOG=debug RUST_BACKTRACE=1` as the environment variable to enable detailed logs.

1. Onboarding mrc to Relay chain

    ```bash
    ./scripts/init.sh onboard-mrc
    ```

1. Parachain will be available at  port `$((9946 + $para_id))` which in default case is `11946`

1. Link to parachain [dashboard](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A11946)

### Generating a new spec file

1. Update `node/chain_spec.rs` with required spec config, defaults to `para_id:2000` and relay chain to be `rococo_local.json` with `protocol_id:mrc-local`
2. Run `cargo run --release build-spec --disable-default-bootnode --chain > ./res/genesis/mrc-spec-rococo.json` to export the chain spec
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

## Additional Resources

- [Cumulus Tutorial](https://docs.substrate.io/tutorials/v3/cumulus/start-relay/)
- [Private Network](https://docs.substrate.io/tutorials/v3/private-network/)
