# 📄 Boosting Rewards Extension

## 📚 Context and Scope

This design introduces a new boosting option in the Frequency runtime. It defines a mechanism for locking tokens that is both time-sensitive and event-driven, with immutable thawing behavior.

While the core `Provider Boosting` system remains unchanged, this proposal builds upon it to incentivize long-term token commitments. Users who participate in extended locks will receive differentiated rewards via minimal extensions to the existing `capacity` pallet.

## ❗ Problem Statement

Business requirements call for a mechanism that allows users to:

- Lock tokens before a **Precipitating Tokenomic Event (PTE)**.
- Unlock tokens over time based on rules triggered by the PTE.
- Maintain **immutability** of the unlock schedule—even governance cannot change it.
- Reclaim tokens safely if the PTE never occurs (via a failsafe).
- Earn **additional rewards** as compensation for committing to extended lock periods.

### Additional Constraints

- The PTE is a **Governance action**.
- Users **must lock** tokens **prior** to the PTE.
- If the PTE is not triggered by a set deadline, tokens should become fully unlockable (failsafe).
- The system must **encourage participation** through enhanced rewards.

## 🎯 Goals and Non-Goals

### Goals

- Introduce **event-driven, schedule-based unlocking**.
- Add **new boosting programs** using a new `StakingType`.
- Provide an **immutable** thaw schedule post-PTE.
- Maintain reward disbursement through a custom `RewardsProvider`.
- Allow users to **opt-in** to different boosting programs.

### Non-Goals

- No UI or wallet-level implementation.
- No parameter consolidation for Provider Boosting.

## ✨ Summary

This document proposes an enhancement to the `Provider Boosting` feature in the `capacity` pallet and the introduction
of a new `boosting program` types to handle creation of programs with new structures. The objective is to support tokens locked with specific
thawing behavior, with such tokens participating in Provider Boosting at a distinct reward rate.

### Key Terms

- **Flexible Boosting**: The current and default reward program.
- **Committed Boosting**: New reward program with new rules.
- **Precipitating Tokenomic Event (PTE)**: Governance-triggered event that starts unlock schedule for Committed Boosting.
- **Pre-PTE Phase**: Phase before the PTE is set by governance.
- **Commitment Phase**: Phase after PTE is triggered where the tokens are 100% frozen.
- **Staged Release Phase**: Phase with a gradual decrease in the commitment requirement phase that allows greater and greater percentage of the committed amount to be unstaked.
- **Expired Phase**: After the Release Phase is completed and all tokens can be unlocked; rewards return to the flexible model.

### Extended Boosting Phases

| Phase               | Can Join | Can Unstake | Unstake Amount | Reward Type       |
|---------------------|----------|-------------|----------------|-------------------|
| Pre-PTE Commitment  | ✅       | 🚫          | 0%             | Committed Rewards |
| Post-PTE Commitment | ✅       | 🚫          | 0%             | Committed Rewards |
| Staged Release      | 🚫       | ✅          | Formula-Based  | Committed Rewards |
| Expired             | 🚫       | ✅          | 100%           | Flexible Rewards  |
| Failsafe Trigger    | 🚫       | ✅          | 100%           | Flexible Rewards  |

## 📂 Capacity Pallet Changes

### Storage and Constants:

```rust
pub enum StakingType {
    MaximumCapacity,
    ProviderCommitedBoost, // RENAME from ProviderBoost
    ProviderFlexibleBoost, // NEW for the default
}

pub const COMMITTED_BOOST_FAILSAFE_UNLOCK_BLOCK_NUMBER: BlockNumber = <some constant>;

#[pallet::storage]
pub type PrecipitatingEventBlockNumber<T: Config> = StorageValue<_, T::BlockNumber, OptionQuery>;
```

- `PrecipitatingEventBlockNumber`: Set by governance to signal a precipitating event.
- `COMMITTED_BOOST_FAILSAFE_UNLOCK_BLOCK_NUMBER`: Block after which full unlock is allowed if PTE does not occur.


### Reward Parameters for Committed Boosting

- `RewardPercentCap` from `Permill::from_parts(5_750);` to `Permill::from_parts(TBD);`

#### Notes

- This could be a new `RewardsProvider` implementation; however, it MUST NOT be possible to exceed the `RewardPoolPerEra`.
- It is possible that the RewardPool is saturated such that the `RewardPercentCap` does NOT apply to those with the higher `RewardPercentCap`, but DOES to those with the lower `RewardPercentCap`. In this case, the entire `RewardPoolPerEra` would NOT be given away. This is an acceptable, but rare outcome.
    - Simple Example:
        - Reward Pool: 9
        - Total Stake: 30
        - Staker 1: 10 @ 50% cap boost type, Reward determined by Stake Percentage 33%, gets 3 Tokens
        - Staker 2: 10 @ 50% cap boost type, Reward determined by Stake Percentage 33%, gets 3 Tokens
        - Staker 3: 10 @ 10% cap boost type, Reward determined by Cap 10%, gets 0.9 Tokens
        - Total Reward Pool not distributed: 2.1 Tokens

### Commitment Release Parameters:

```rust
/// Number of epochs after the `PrecipitatingEventBlockNumber` that no unstaking is allowed
pub const COMMITMENT_EPOCHS: u32 = < some constant>;
/// Number of epochs after the `COMMITMENT_EPOCHS` that restrict the unstake amount
pub const COMMITMENT_RELEASE_EPOCHS: u32 = < some constant>;
```

**Unlock formula per era:**

```text
If current_era < COMMITMENT_EPOCHS:
    unlock_ratio = 0
Else:
    thaw_era = current_era - INITIAL_FREEZE_THAW_ERAS
    unlock_ratio = 1 / (COMMITMENT_RELEASE_EPOCHS - min(thaw_era, COMMITMENT_RELEASE_EPOCHS) + 1)
```

### Optional Optimizations
- Instead of having the PTE set to the PTE, it could instead be set to the `COMMITMENT_EPOCHS` value and remove the `COMMITMENT_EPOCHS` value entirely. Governance could do the calculation of the PTE plus the `COMMITMENT_EPOCHS` and just set an `CommittedBoostThawStartBlockNumber`.
- The `CommittedBoostThawStartBlockNumber` or `PrecipitatingEventBlockNumber` could be set via upgrade migration to be the same as the fallback value, although that would then not have the immediate 100% unlock ratio

### Additional Related Capacity Changes

- Increase `MaxUnlockingChunks` to `40` to accommodate extended unlocks.
- Adjust RewardPercentCap:
    - From: `Permill::from_parts(5_750);` 0.575%
    - To: `Permill::from_parts(3_833);` 0.3833%
    - Rationale: Larger participation expected, but lower per-user reward.

## 🔐 Governance Integration

- Governance sets the `PrecipitatingEventBlockNumber`.
- Governance cannot modify thaw rules or formulas once the PTE occurs.

## PTE Failsafe

If the PTE is never triggered before the failsafe block, the program automatically expires:

- Rewards revert to default.
- Thaw ratio becomes 100%.
- Users can unstake fully.

## 🔄 Migration Strategy

All current ProviderBoost participants will be migrated to ProviderCommitedBoost during rollout.

Users who wish to opt-out must unstake before the upgrade.

## Example User actions for the Committed Boosting Program

1. Stake
    - Before the upgrade (to be migrated and then deprecated): `capacity.provider_boost(target, amount)`
    - After the upgrade: `capacity.provider_boost_v2(target, amount, type)`
2. Claim Rewards
3. (Governance) PTE Happens, Time Passes
4. (If desired) Commitment Release Period: User requests a partial unstake: `capacity.unstake(limited_amount)`
5. (If desired) Post Expiration: Unstake up to full amount: `capacity.unstake(full_amount)`
