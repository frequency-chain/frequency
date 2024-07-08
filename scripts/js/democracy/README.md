# Local Runtime Upgrade via Referendum

If you are trying to enact a runtime upgrade via Polkadot's referenda process, keep in mind the following workflow:

1. Runtime upgrade is authorized
2. A proposal for that upgrade is made
3. The proposal is endorsed by an actor on the network
4. The proposal is tabled for referendum
5. A majority of actors vote "Aye" on the referendum
6. The referendum passes
7. The upgrade is enacted

To accomplish the above workflow, you will need to take the following steps:

### Upgrade Authorization

1. Build the WASM for the runtime upgrade:

```
make build
make specs-testnet-2000
```

This will place a WASM in the `res/genesis/testnet` folder.

Once you have the WASM, submit a preimage for it using the `parachainSystem.authorizeUpgrade` extrinsic. If you are using the Polkadot JS UI, you can access a widget for doing this in the [democracy panel](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/democracy).

The upgrade should take about a block to register on the chain (take note of the chain event stream for a `democracy.PreimageNoted` event). Grab the hash from the event.

### Proposal

To make a proposal, you can use the `propose` function located at `scripts/js/democracy/submitProsal.js`. Pass the hash from the previous step into this function to motion for a proposal. You should see a `democracy.Proposed` event appear in the event stream.

### Endorsement

For the proposal to proceed to the referendm stage, it must be endorsed. In Polkadot parlance, this is called "Secconding". To endorse your proposal, use the `endorse` function located in the `scripts/js/democracy/endorseProposal.js` file. Pass it the index of the proposal you want to endorse (you can get the index from the democracy panel or from the `democracy.Proposed` event).

### Voting

At the end of the launch period, the endorsed proposal will move to to referendum stage. Here, the proposal will need to get a majority of "Aye" votes to be enacted. Keep in mind here that votes equate with balance.

To vote for a proposal, get the `refIndex` from the `democracy.Started` event and pass it to the `aye` or `nay` function from `scripts/js/democracy/voteOnReferendum.js`. You may pass in a balance as well (this is the value correlated with one's vote).

### Enactment

Once the referendum passes, you should see a `democracy.Passed` event as well as a `scheduler.Scheduled` event containing a block number where the upgrade will be authorized (usually 5 blocks into the future). When that block number is reached, you should see a `parachainSystem.UpgradeAuthorized` event appear in the event stream. Once you do, it is time to enact your upgrade.

This is an action you will need to do as _sudo_. Visit the [extrinsics panel](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/extrinsics) and first choose the `sudo.UncheckedWeight` extrinsic. Nested underneath it, choose `parachainSystem.enactAuthorizedUpgrade` and upload the same WASM you authorized earlier. Submit. If all goes well, you will see a successful `sudo.Sudid` event (`Ok`), with a `parachainSystem.ValidationFunctionApplied` event soon after (about 2 blocks). If you do, your upgrade was successful.
