# HOW TO: Do Forkless Upgrades
### via Sudo+Polkadot dashboard

To upgrade a parachain, the relay chain has to know about the upgrade in advance.  Then when the enactment extrinsic for the upgrade is submitted, the relay chain is also alerted and checks the WASM against the previously provided hash.

The extrinsics are part of the cumulus pallet_parachain_system.  Both extrinsics call ensureRoot on the origin, so only Sudoers can call it.

## Setup:
1. Build the new release target
2. If developing locally, ensure that the two relay nodes and the parachain are running
3. Connect to the [Polkadot dashboard](https://polkadot.js.org/apps/#/explorer)
4. Depending on the chain you’re using,
   1. If on a rococo parachain, for example, you must have imported the Sudo account keys into the [list of connected accounts](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Ffrequency-rococo-rpc.polkadot.io#/accounts)  for this to work.
   2. If developing locally you will be using whatever root key is configured in the `.env` file. If you did not change it from `.env.sample`, this is the **Alice** account.

## Authorize the upgrade
This step tells the relay chain to expect an upgrade and the expected hash of the upgrade.

1. Go to the **Developer → Extrinsics** panel.
2. Select the Sudo account for “**using the selected account**” from the dropdown. The Sudo account must already have been imported as an account into the dashboard.
3. From “**submit the following extrinsic**”, select “**parachainSystem**” and “**authorizeUpgrade**”
4. Toggle “**hash a file**” switch
5. Click on the form entry to open a file browsing window
6. Select the new WASM `frequency/target/release/wbuild/frequency-runtime/frequency_runtime.compact.compressed.wasm`  (or where it’s located otherwise). The rest of the fields should be populated with the hash data and encoding details.
7. Click “**Submit Transaction**” and sign the transaction using the Sudo account key.

## Schedule the enactment of the upgrade
This step actually performs the forkless upgrade.  If you are running a local parachain + relay, you don’t need to schedule the upgrade; run the upgrade script, which directly submits “enactAuthorizedUpgrade” as an RPC call by using the configured root key, without scheduling it.

1. From the same panel (**Developer → Extrinsics**), in “**submit the following extrinsic**,” select “**scheduler**” and then “**schedule**”
2. Skip past everything else and in “**Value: Call**” select “**parachainSystem**” and “**enactAuthorizedUpgrade(code)**”
3. Toggle “**file upload**” on.
4. Select the new WASM, located in the same spot as before, `frequency/target/release/wbuild/frequency-runtime/frequency_runtime.compact.compressed.wasm` or wherever it's located.
5. Choose the block number in the future desired, typically at least 5 minutes in the future, or 300 blocks if the blocks are 6 s each.
6. Click “**Submit Transaction**” and sign the transaction as before, using the Sudo account key.
