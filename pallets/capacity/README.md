# Capacity Pallet

The Capacity Pallet manages the staking and balances for Capacity, an alternative payment system on Frequency.

## Summary

Capacity is an alternative to paying with tokens for a limited set of calls.
These Capacity eligible extrinsics are noted in each pallet's documentation with "Capacity" in the Payment column of the extrinsics section.
Tokens can be staked to generate Capacity for a targeted Provider.
The generated Capacity renews each [Epoch](#capacity-epoch).
[Learn more about Capacity](https://docs.frequency.xyz/Tokenomics/ProviderIncentives.html#capacity-model).

### Staking & Unstaking
Currently, the token to Capacity ratio is 50:1.
For example, for a 5 token stake, a Provider would receive 0.1 Capacity.
Staking and unstaking affect available Capacity immediately.

### Capacity Epoch

A Capacity Epoch is a period consisting of a specific number of blocks, during which a Provider's utilization of network Capacity is capped at the amount of generated Capacity targeted to that Provider.
At the start of each new Epoch, the available Capacity is renewed for each Provider, regardless of how much they consumed in the prior Epoch.
The duration of a Capacity Epoch is determined by Governance, and is currently set to 7200 blocks.
With the current average block time of approximately 12 seconds, one Capacity Epoch lasts around 24 hours on Mainnet.

### Thaw Period

After unstaking, the tokens will still be frozen for a set amount of time before they are unencumbered and able to be transferred.
The `UnstakingThawPeriod` constant defines the number of Epochs that must pass before the tokens may be reclaimed for any use via `withdrawUnstaked()`.
Currently it is set to 30 Epochs or ~30 days after unstaking.

### Actions

The Capacity Pallet provides for:

- Staking to receive Capacity
- Unstaking & Thaw Period
- Capacity Epoch management

## Interactions

### Extrinsics

| Name/Description                 | Caller        | Payment | Key Events                                                                                                    | Runtime Added |
| -------------------------------- | ------------- | ------- | ------------------------------------------------------------------------------------------------------------- | ------------- |
| `stake`<br />Lock tokens to grant Capacity to a Provider | Token Account | Tokens | [`Staked`](https://frequency-chain.github.io/frequency/pallet_capacity/pallet/enum.Event.html#variant.Staked) | 1             |
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
