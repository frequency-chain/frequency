Running E2E Tests
=========================

To run all tests (and automatically start up a Frequency node):

`make e2e-tests`

To run all tests (after starting up a Frequency node):

`npm run test`

To run an individual test (after starting up a Frequency node):

Note: this is for the "createMsa" tests

`npm run test:serial -- --grep createMsa`

See below for running load tests and relay chain tests.

Waiting for Finalization
============================

Some actions require waiting for finalization before the next action can happen.

Example: Funding a key that will then take an action that uses that funding.

While sometimes this is not needed, it creates a race condition and sometimes the transaction pool will reject the
second transaction as it is unfunded.

The transaction pool doesn't know when two transactions are dependent if they are not from the same account with the
same nonce.

Make sure to wait for finalization before attempting the next dependent transaction.

Notes on E2E Testing
============================

1. Avoid using anonymous arrow functions for test blocks (`describe`, `it`, `before`, etc).
Though technically legal, it's discouraged in Mocha. See [here](https://mochajs.org/#arrow-functions) for details.
2. Avoid using the standard Substrate dev accounts ('//Alice', '//Bob', etc) for running tests. It causes interference
when multiple test suites are running in parallel, or against the same Frequency chain. Instead, create a new account & MSA
for each test and fund it from one of the well-known dev accounts. There are helper functions to assist in this, like so:
```
/* In suite initialization */
const keypair = createAndFundAccount(); // Creates keypair and transfers existential deposit into it.
await ExtrinsicHelper.createMsa(keypair).fundAndSend(); // see below for more details
```
3. There are two types of error that can result from a call to an extrinsic using polkadot.js. The first is an `RpcError`
thrown due to some fundamental failure. The second is an `ExtrinsicFailed` event present in the returned event stream. The
extrinsic helper library parses the event stream and converts a `DispatchError` present in the stream into a thrown `EventError`.
Extrinsics which fail and include an `ExtrinsicFailed` event in the stream should also have a `DispatchError` present, and hence
it is not necessary to look for the `ExtrinsicFailed` event, but simply to expect the extrinsic call to throw (rejected Promise)
or not.
4. There are 2 environment variables that can be set:
    `VERBOSE_TESTS`: 'true' or '1' enables verbose logging in tests using the `log()` function
    `WS_PROVIDER_URL`: allows override of the default Frequency URL (localhost)
5. ExtrinsicHelpers: this is a static class that is initialized by the root hooks. Each helper method returns an Extrinsic object
with the following methods:
    - getEstimatedTxFee(): Get payment info for the extrinsic call represented by the current object
    - fundOperation(): Call getEstimatedTxFee() to estimate fee, and transfer tokens into the current account from one of the pre-funded dev accounts
    - signAndSend(): Sign & submit the extrinsic call represented by the current object to the chain
    - fundAndSend(): Combines fundOperation() and signAndSend()
6. Expiration block numbers
    Rather than hard-coding block number expirations into test cases, it's better to query the last block in the chain for the current
    block number & then add or subtract as the use case dictates.

    EXAMPLE:
    ```
            const payload = {
            authorizedMsaId: providerId,
            schemaIds: [schemaId],
            expiration: (await ExtrinsicHelper.getLastBlock()).block.header.number.toNumber() + 5,
        }
    ```
7. Extrinsic helper methods & returned events
Many of the extrinsic helper methods pass an event type to the underlying Extrinsic object that can be used to parse the targeted event type
from the resulting stream and return it specifically. The `Extrinsic::signAndSend()` method returns an object of `{ target, eventMap }` where
`target` will be the parsed target event *if present*. The `eventMap` is a map of <string, event> with the keys being `paletteName.eventName`.
A special key "defaultEvent" is added to also contain the target event, if present.
Events may be used with type guards to access the event-specific structure. Event types are in the `ApiRx.events.<palette>.*` structure, and can be
accessed like so:
    ```
    const extrinsic = ExtrinsicHelper.createMsa(keypair);
    const { target: targetEvent, eventMap } = await extrinsic.fundAndSend();
    if (targetEvent) {
        const msaId = targetEvent.data.msaId;
    }
    // OR null coalescing
    const maybeMsaId = targetEvent?.data.msaId;

    // OR Throw unless defined
    const throwIfNotMsaId = targetEvent!.data.msaId;
    ```

Load Testing
==================
Load tests are located in the `load-tests/` directory.
The tests in that folder are NOT run with a normal test run.
It is configured to run in manual sealing mode only. To run the tests, do the following:

```
make e2e-tests-load
```

That make command does approximately the following:

1. Start the chain in manual sealing mode
```
make start-manual
```

2. Run tests
```
cd e2e
npm run test:load
```

Relay Chain Testing
==================
There are a few tweaks to running the tests against the relay chain.

1. The funding for the tests can be in a separate command
2. The tests can be run entirely separately

Run the tests all in parallel
```
npm run test:relay
```

Just do the funding
```
npm run test:relay-funding
```

Run just one test
```
npm run test:relay-single <test file>
```

See `.github/workflows/e2e-testnet.yml` for how it works in CI
