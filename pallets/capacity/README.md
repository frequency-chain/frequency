# Capacity Pallet

Manages the staking and balances for Capacity, an alternative payment system on Frequency.

## Summary

Capacity is an alternative payment system for a subset of calls that renews each Epoch.
Tokens staked create Capacity for a targeted Provider.
[Learn more about Capacity](https://docs.frequency.xyz/Tokenomics/ProviderIncentives.html#capacity-model).

### Staking & Unstaking
Currently, the token to Capcity ratio is 50:1.
For example, for a 5 token stake, a Provider would receive 0.1 Capacity.
Staking and unstaking affect available Capacity immediately.

### Capacity Epoch

A Capacity Epoch is the number of blocks before Capacity balances are able to be renewed.
The value is managed by Governance, and is currently set to ~24 hours.

### Thaw Period

After unstaking, the tokens will still be frozen for a set amount of time before they are unencumbered and able to be transferred.
The `UnstakingThawPeriod` constant defines the number of Epochs that must pass before the tokens may be reclaimed for any use via `withdrawUnstaked()`.
Currently it is set to 30 Epochs or ~30 days after unstaking.

### Actions

The Capacity pallet provides for:

- Staking to receive Capacity
- Unstaking & Thaw Period
- Capacity Epoch management

## Interactions

### Extrinsics

| Name/Description                 | Caller        | Payment | Key Events                                                                                                    | Runtime Added |
| -------------------------------- | ------------- | ------- | ------------------------------------------------------------------------------------------------------------- | ------------- |
| `stake`<br />Lock tokens to gain Capacity | Token Account | Tokens | [`Staked`](https://frequency-chain.github.io/frequency/pallet_capacity/pallet/enum.Event.html#variant.Staked) | 1             |
| `unstake`<br />Begin the process of unlocking tokens by unstaking currently staked tokens | Token Account | Tokens | [`UnStaked`](https://frequency-chain.github.io/frequency/pallet_capacity/pallet/enum.Event.html#variant.UnStaked) | 1             |
| `withdraw_unstaked`<br />Complete the process of unlocking tokens staked by releasing locks on expired unlock chunks | Token Account | Tokens | [`StakeWithdrawn`](https://frequency-chain.github.io/frequency/pallet_capacity/pallet/enum.Event.html#variant.StakeWithdrawn) | 1             |

See [Rust Docs](https://frequency-chain.github.io/frequency/pallet_capacity/pallet/struct.Pallet.html) for more details.

### State Queries

| Name      | Description         | Query                    | Runtime Added |
| --------- | ------------------- | ------------------------ | ------------- |
| Get Capacity Ledger | Returns the Capacity balance details for a Provider's MSA Id  | `capacityLedger` | 1             |
| Get Current Epoch | Returns the current Capacity Epoch number  | `currentEpoch` | 1             |
| Get Current Epoch Info | Returns information about the current Capacity Epoch such as the starting block number | `currentEpochInfo` | 1             |
| Get Staking Account Ledger | Returns information about an account's current staking details | `stakingAccountLedger` | 1             |
| Staking Target Ledger | Returns information about an account's current staking details for a specific target Provider MSA Id | `stakingTargetLedger` | 1             |
| Get Unstake Information | Returns the information about an account's current unstaking details and the unlocking chunks | `unstakeUnlocks` | 1             |


See the [Rust Docs](https://frequency-chain.github.io/frequency/pallet_capacity/pallet/storage_types/index.html) for additional state queries and details.
