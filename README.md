Frequency is a Polkadot parachain designed to run Decentralized Social Network Protocol (DSNP), but it could run other things.

# Table of Contents

-   [Prerequisites](#prerequisites)
-   [Build](#build)
    -   [Local desktop](#local-desktop)
    -   [Remote Instance](#remote-instance-such-as-aws-ec2)
-   [Run](#run)
    -   [Collator Node in Instant Sealing Mode](#1-collator-node-in-instant-sealing-mode)
    -   [Collator Node with Local Relay Chain](#2-collator-node-with-local-relay-chain)
-   [Run Tests](#run-tests)
-   [Run Benchmarks](#run-benchmarks)
-   [Generate Specs](#generate-specs)
-   [Format, Lint and Audit Source Code](#format-lint-and-audit-source-code)
-   [Verify Runtime](#verify-runtime)
-   [Local Runtime Upgrade](#local-runtime-upgrade)
-   [Contributing](#contributing)
-   [Additional Resources](#additional-resources)
-   [Miscellaneous helpful commands](#miscellaneous)

# Prerequisites

1. [Docker Engine](https://docs.docker.com/engine/install/)\*
1. [Docker Compose](https://docs.docker.com/compose/install/)

---

-   For Mac users, [Docker Desktop](https://docs.docker.com/desktop/mac/install/) engine also installs docker compose environment, so no need to install it separately.

## Hardware

We run benchmarks with and recommend the same [reference hardware specified by Parity](https://wiki.polkadot.network/docs/maintain-guides-how-to-validate-polkadot#reference-hardware).

# Build

## Local Desktop

1. Install Rust using the [official instructions](https://www.rust-lang.org/tools/install).
2. Check out this repository
3. Initialize your Wasm Build environment:
    ```sh
    cd [path/to/frequency]
    ./scripts/init.sh install-toolchain
    ```
4. Build Wasm and native code.

    _Note, if you get errors complaining about missing
    dependencies (cmake, yarn, node, jq, etc.) install them with your favorite package
    manager(e.g. Homebrew on Mac) and re-run the command again._

    ```sh
    cargo build --release
    ```

    Alternatively you may run `TARGET=build-node ./ci/build.sh`

At this point you should have `./target/release` directory generated locally with compiled
project files.

### asdf Support

Install the required plugins for asdf:

```sh
asdf plugin-add rust
asdf plugin-add make
asdf plugin-add cmake [https://github.com/srivathsanmurali/asdf-cmake.git](https://github.com/srivathsanmurali/asdf-cmake.git)
```

Install the dependency versions declared in `.tool-versions`

```sh
asdf install
```

NOTE: I could find no clang plugin that worked so your system still needs clang to be installed.

## Remote Instance such as AWS EC2

For remote instances running Linux, if you want to check out and build such as on an AWS EC2 instance, the process is slightly different to what is in the [Substrate documentation](https://docs.substrate.io/main-docs/install/linux/).

### Ubuntu

1. Upgrade the instance and install missing packages with `apt`:

```bash
sudo apt upgrade
sudo apt upgrade git
sudo apt install —assume-yes build-essential
sudo apt install --assume-yes clang curl libssl-dev cmake
```

2. Follow [official instructions to install rust](https://www.rust-lang.org/tools/install), but select `3. customize the installation`, then reply **n** to `Modify PATH variable? (Y/n)`
3. Follow steps 6-10 at [Substrate: Linux development](https://docs.substrate.io/main-docs/install/linux/)
4. Proceed with checking out and building frequency as above.

# Run

There are 2 options to run the chain locally:

1.  Collator Node in Instant Sealing Mode,
1.  Collator Node with Local Relay Chain

## 1. Collator Node in Instant Sealing Mode

![](docs/images/local-dev-env-option-1.jpg)

This option runs just one collator node in instant seal mode and nothing more.
A "collator node" is a Frequency parachain node that is actively collating (aka forming blocks to submit to the relay chain). The instant seal mode allows a blockchain node to author a block
as soon as it goes into a queue. This is also a great option to run with an example client.

### In Terminal

```sh
make start
```

### In Docker Container

```sh
docker run --rm -p 9944:9944 -p 9933:9944 -p 30333:30333 frequencychain/instant-seal-node
```

To stop running chain hit [Ctrl+C] in terminal where the chain was started.

| **Node**                |             **Ports**             | **Explorer URL**                                                                          |
| ----------------------- | :-------------------------------: | ----------------------------------------------------------------------------------------- |
| Frequency Collator Node | ws:`9944`, rpc`:9933`, p2p:`3033` | [127.0.0.1:9944](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/explorer) |

## 2. Collator Node with Local Relay Chain

![](docs/images/local-dev-env-option-2.jpg)

### Mixed Terminal/Docker

This option runs one collator node as local host process and two relay chain validator nodes in each own docker container.

1. Start relay chain validator nodes.

    ```sh
    make start-relay
    ```

1. Register a new parachain slot (parachain id) for Frequency. _Note, if parachain was
   previously registered on a running relay chain and no new registration is required,
   then you can skip the above step._
    ```sh
    make register
    ```
1. Generate chain spec files. If this is your first time running the project or
   new pallets/runtime code changes have been made to Frequency, then the chain specs
   need to be generated. Refer to [generation spec file](#generate-a-new-spec-file)
   for more details.

1. Start Frequency as parachain. This step will generate genesis/wasm and onboard the
   parachain.

    ```sh
    make start-frequency
    ```

1. Onboard Frequency to the relay chain
    ```sh
    make onboard
    ```

#### Stop and Clean Environment

1. Off-board Frequency from relay chain.: `make offboard`
2. to stop Frequency running in the terminal: `[Ctrl+C] `
3. Stop the relay chain. `make stop-relay`
4. Run to remove unused volumes. `make docker-prune`
5. Clean up temporary directory to avoid any conflicts with next onboarding:
   `rm -fr /tmp/frequency`

### All in Docker Container

:exclamation: Currently does not work on M\* series MacOS laptops. See https://github.com/LibertyDSNP/frequency/issues/432

Start:

```sh
make start-frequency-docker
```

Stop:

```sh
make stop-frequency-docker
```

| **Node**             | **Ports**                           | **Explorer URL**                                                                          |
| -------------------- | ----------------------------------- | ----------------------------------------------------------------------------------------- |
| Frequency Relay Node | ws:`9944`, rpc`:9933`, p2p:`30333`  | [127.0.0.1:9944](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/explorer) |
| Alice Relay Node     | ws:`:9946`, rpc`:9935`, p2p:`30335` | [127.0.0.1:9946](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9946#/explorer) |
| Bob Relay Node       | ws:`:9947`, rpc`:9936`, p2p:`30336` | [127.0.0.1:9947](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9947#/explorer) |

## Run Tests

```sh
# Activate selected features
cargo test --features runtime-benchmarks, std

# Activate all features and test all packages in the workspace
cargo test --all-features --workspace --release
```

## Generate Specs

To build spec against specific chain config specify chain name in the command above.

1. Update `node/**/chain_spec.rs` with required spec config, defaults to `para_id:2000`
   and relay chain to be `rococo_local.json` with `protocol_id:frequency-local`
1. Export the chain spec
    ```sh
    cargo run --release build-spec --disable-default-bootnode > ./res/genesis/local/frequency-spec-rococo.json
    ```
1. Export the raw chain spec
    ```sh
    cargo run --release build-spec --raw --disable-default-bootnode --chain ./res/genesis/local/frequency-spec-rococo.json > ./res/genesis/local/rococo-local-frequency-2000-raw.json
    ```

Alternatively, run the following to generate plain and raw frequency spec along with
genesis state and WASM:

-   `make specs-rococo-2000` for Rococo local testnet
-   `make specs-rococo-4044` for Rococo public testnet

# Format, Lint and Audit Source Code

-   Format code with `make format` according to style guidelines and configurations in `rustfmt.toml`.
-   Lint code with with `make lint` to catch common mistakes and improve your [Rust](https://github.com/rust-lang/rust) code.
-   Alternatively, run `make format-lint` to run both at the same time.
-   Run `cargo-audit` to audit Cargo.lock files for crates with security vulnerabilities reported to the RustSec Advisory Database. [See cargo-audit installation instructions](https://github.com/RustSec/rustsec/tree/main/cargo-audit)

# Run Benchmarks

In order to achieve a certain block time, for example 6 seconds per block,
there is a limited number of lines of code can be run during that time frame.
When writing a pallet function, the developer is responsible for calculating its
computational complexity, which is called **weight**. The process of
determining that complexity, or simply put, time cost is called benchmarking.

There are 2 options to run benchmarks:

-   Locally on a developer laptop
-   Remotely on referenced hardware in the cloud

## 1. Locally

```sh
make benchmarks
```

## 2. On Referenced Hardware

:exclamation: DO NOT commit weights yourself!
They will be auto-committed by CI job upon completion.

To trigger running benchmarks on reference hardware in the cloud:

1. Enter `/run-benchmarks` slash command in a GitHub PR comment.
   This will kick off remote Jenkins job.
1. Upon completion, the updated weights will be committed back to the PR branch in a new commit.

# Verify Runtime

1. Check out the commit at which the runtime was built.
2. Use srtool to verify the runtime:
    ```sh
    TARGET=build-runtime RUST_TOOLCHAIN=nightly ./ci/build.sh
    ```

# Local Runtime Upgrade

To upgrade the runtime, run the following command:

```sh
make upgrade-local
```

The current scripts follow this process for upgrading locally:

1. Build new runtime and generate the compressed wasm
2. Call `authorizeUpgrade` extrinsic from parachain system to initiate the upgrade.
3. Call `enactAuthorizedUpgrade` extrinsic from parachain system to enact the upgrade.
4. For testnet and mainnet, the upgrade is done slightly differently using `scheduler` and enactment is scheduled for a specific block number in the future.

# Contributing

Interested in contributing?
Wonderful!
Please check out [the information here](./CONTRIBUTING.md).

# Additional Resources

-   [Cumulus Project](https://github.com/paritytech/cumulus)
-   [Cumulus Tutorials](https://docs.substrate.io/tutorials/)
-   [Prep Substrate environment for development](https://docs.substrate.io/install/)

# Miscellaneous

```sh
# Clean up local docker resources
make docker-prune

# View all listening ports
lsof -i -P | grep -i "listen"

# View ports Frequency node is listening on
lsof -i -P | grep -i "listen" | grep frequency
```
