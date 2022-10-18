# HOW TO: Do Forkless Upgrades
### via Sudo+Polkadot dashboard

To upgrade a parachain, the relay chain has to know about the upgrade in advance.  Then when the enactment extrinsic for the upgrade is submitted, the relay chain is also alerted and checks the WASM against the previously provided hash.

The extrinsics are part of the cumulus pallet_parachain_system.  Both extrinsics call ensureRoot on the origin, so only Sudoers can call it.

## Setup:
1. Build the new release target
    a. Local
        i. If developing locally, ensure that the two relay nodes and the parachain are running
        i. If developing locally, ensure that the two relay nodes and the parachain are running
        ii. Run `make upgrade-local` from the root frequency directory. The make command will execute a script that builds the release target for local development, calls the extrinsic to authorize the upgrade, then calls the extrinsic to enact the upgrade. No further steps are required to upgrade the local runtime
    b. Testnet (staging): Run `make build-rococo` from the root frequency directory.
    b. Mainnet (production): Run `make build-mainnet` from the root frequency directory.
3. Connect to the [Polkadot dashboard](https://polkadot.js.org/apps/#/explorer)
4. Depending on the chain you’re using,
   1. If on a rococo parachain, for example, you must have imported the Sudo account keys into the [list of connected accounts](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Ffrequency-rococo-rpc.polkadot.io#/accounts)  for this to work.
   2. If developing locally you will be using whatever root key is configured in the `.env` file. If you did not change it from `.env.sample`, this is the **Alice** account.

## Authorize the upgrade
This step tells the relay chain to expect an upgrade and the expected hash of the upgrade, effectively scheduling the upgrade with the relay chain. The scheduler pallet is not required to schedule a runtime upgrade. (See https://substrate.stackexchange.com/questions/5356/is-it-possible-to-schedule-a-sudo-upgrade)

1. Go to the **Developer → Extrinsics** panel.
2. Select the Sudo account for “**using the selected account**” from the dropdown. The Sudo account must already have been imported as an account into the dashboard.
3. From the same panel (**Developer → Extrinsics**), in “**submit the following extrinsic**,” select “**sudo**” and then “**sudo**”
4. From “**call: Call**”, select “**parachainSystem**” and “**authorizeUpgrade**”
5. Toggle “**hash a file**” switch
6. Click on the form entry to open a file browsing window
7. Select the new WASM
    a. Testnet (staging) `frequency/target/production/wbuild/frequency-rococo-runtime/frequency_runtime.compact.compressed.wasm`
    b. Mainnet (production) `frequency/target/production/wbuild/frequency-runtime/frequency-runtime.compact.compressed.wasm`
    *Note: The production directory in the paths to each WASM means that the production build profile is used in generating each one. See the root `Cargo.toml`.
    **The rest of the fields in the Polkadot JS UI should be populated with the hash data and encoding details.
8. Click “**Submit Transaction**” and sign the transaction using the Sudo account key.

## Enactment of the upgrade
This step actually performs the forkless upgrade by submitting “enactAuthorizedUpgrade” as an RPC call by using the configured root key.

1. From the same panel (**Developer → Extrinsics**), in “**submit the following extrinsic**,” select “**sudo**” and then “**sudo**”
2. In “**call: Call**”, select “**parachainSystem**” and “**enactAuthorizedUpgrade(code)**”
3. Toggle “**file upload**” on.
4. Select the new WASM, located in the same spot as before.
5. Click “**Submit Transaction**” and sign the transaction as before, using the Sudo account key.
