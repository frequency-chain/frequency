# Time-Release Pallet

Provides a way to schedule a balance freeze on an account.

## Summary

The `time-release` pallet employs a _graded release_ approach, which thaws a specific amount of balance every period of time, until all balance is thawed.

Note: This pallet is a fork and modification of the [ORML-vesting](<[vesting](https://github.com/open-web3-stack/open-runtime-module-library/tree/master/vesting)>).

### Release Schedule

The schedule of a release on hold is described by the data structure `ReleaseSchedule`. Starting from the specified block number denoted as `start`, the schedule operates on a periodic basis. For each `period` amount of blocks, a designated `per_period` amount of balance is unfrozen. This process continues until the specified number of periods, denoted as `period_count`, is reached. It's important to highlight that in release schedules, the concept of time is measured in terms of block numbers. Accessing all `ReleaseSchedule` instances associated with an account is possible through querying the chain state.

### Actions

The Time-Release pallet provides for:

- Creating a transfer with a schedule for release
- Claiming balances that are released
- Governance updates of schedules

## Interactions

### Extrinsics

| Name/Description                                                                            | Caller            | Payment | Key Events                                                                                                                                          | Runtime Added |
| ------------------------------------------------------------------------------------------- | ----------------- | ------- | --------------------------------------------------------------------------------------------------------------------------------------------------- | ------------- |
| `transfer`<br />Transfer tokens to another account with an unlock schedule                  | Token Account     | Tokens  | [`ReleaseScheduleAdded`](https://frequency-chain.github.io/frequency/pallet_time_release/pallet/enum.Event.html#variant.ReleaseScheduleAdded)       | 24            |
| `claim`<br />Remove the lock on tokens for the calling account when the schedule allows     | Account with Lock | Tokens  | [`Claimed`](https://frequency-chain.github.io/frequency/pallet_time_release/pallet/enum.Event.html#variant.Claimed)                                 | 24            |
| `claim_for`<br />Remove the lock on tokens for a different account when the schedule allows | Any Token Account | Tokens  | [`Claimed`](https://frequency-chain.github.io/frequency/pallet_time_release/pallet/enum.Event.html#variant.Claimed)                                 | 24            |
| `update_release_schedules`<br />Governance action to update existing schedules              | Governance        | Tokens  | [`ReleaseSchedulesUpdated`](https://frequency-chain.github.io/frequency/pallet_time_release/pallet/enum.Event.html#variant.ReleaseSchedulesUpdated) | 24            |

See [Rust Docs](https://frequency-chain.github.io/frequency/pallet_time_release/pallet/struct.Pallet.html) for more details.

### State Queries

| Name             | Description                                   | Query              | Runtime Added |
| ---------------- | --------------------------------------------- | ------------------ | ------------- |
| Release Schedule | Retrieves the release schedule for an account | `releaseSchedules` | 24            |

See the [Rust Docs](https://frequency-chain.github.io/frequency/pallet_time_release/pallet/storage_types/index.html) for additional state queries and details.
