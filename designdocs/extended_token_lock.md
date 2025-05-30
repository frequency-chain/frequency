# 📄 Token Locking for Extended Boosting Rewards

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
- Allow users to **opt-in** to extended boosting programs.

### Non-Goals

- No UI or wallet-level implementation.
- No parameter consolidation for Provider Boosting.

## ✨ Summary

This document proposes an enhancement to the `Provider Boosting` feature in the `capacity` pallet and the introduction
of a new `boosting program` types to handle creation of programs with new structures. The objective is to support tokens locked with specific
thawing behavior, with such tokens participating in Provider Boosting at a distinct reward rate.

### Key Terms

- **Extended Boosting Program**: New reward program with distinct rules.
- **Precipitating Tokenomic Event (PTE)**: Governance-triggered event that starts unlock schedule.
- **Pre-PTE Freeze**: Lock phase before PTE.
- **Post-PTE Freeze**: Lock phase after PTE but before thawing starts.
- **Extended Thaw**: Gradual unlock phase after freeze ends.
- **Expired Program**: All tokens can be unlocked; normal rewards resume.

### Extended Boosting Phases

| Phase            | Can Join | Can Unstake | Unstake Amount | Stake Reward     |
|------------------|----------|-------------|----------------|------------------|
| Pre-PTE          | ✅       | 🚫          | 0%             | Extended Rewards |
| Post-PTE         | ✅       | 🚫          | 0%             | Extended Rewards |
| Extended Thaw    | 🚫       | ✅          | Formula-Based  | Extended Rewards |
| Expired Program  | 🚫       | ✅          | 100%           | Default Rewards  |
| Failsafe Trigger | 🚫       | ✅          | 100%           | Default Rewards  |

## 📂 Capacity Pallet Changes

### Storage and Constants:

```rust
pub enum StakingType {
    MaximumCapacity,
    ProviderBoost,
    ProviderExtendedBoost, // NEW
}

pub const EXTENDED_BOOST_FAILSAFE_UNLOCK_BLOCK_NUMBER: BlockNumber = <some constant>;

#[pallet::storage]
pub type PrecipitatingEventBlockNumber<T: Config> = StorageValue<_, T::BlockNumber, OptionQuery>;
```

- `PrecipitatingEventBlockNumber`: Set by governance to signal a precipitating event.
- `EXTENDED_BOOST_FAILSAFE_UNLOCK_BLOCK_NUMBER`: Block after which full unlock is allowed if PTE does not occur.


### Reward Parameters for Extended Boosting

- `RewardPercentCap` from `Permill::from_parts(5_750);` to `Permill::from_parts(TBD);`

#### Notes

- This could be a new `RewardsProvider` implementation; however, it MUST NOT be possible to exceed the `RewardPoolPerEra` which a simple adjustment of the cap could enable.

### Thaw Parameters:

```rust
/// Number of epochs after the `PrecipitatingEventBlockNumber` that no unstaking is allowed
pub const INITIAL_FREEZE_THAW_EPOCHS: u32 = < some constant>;
/// Number of epochs after the `INITIAL_FREEZE_THAW_EPOCHS` that restrict the unstake amount
pub const UNLOCK_THAW_EPOCHS: u32 = < some constant>;
```

**Unlock formula per era:**

```text
If current_era < INITIAL_FREEZE_THAW_EPOCHS:
    unlock_ratio = 0
Else:
    thaw_era = current_era - INITIAL_FREEZE_THAW_ERAS
    unlock_ratio = 1 / (UNLOCK_THAW_EPOCHS - min(thaw_era, UNLOCK_THAW_EPOCHS) + 1)
```

### Optional Optimizations
- Instead of having the PTE set to the PTE, it could instead be set to the `INITIAL_FREEZE_THAW_EPOCHS` value and remove the `INITIAL_FREEZE_THAW_EPOCHS` value entirely. Governance could do the calculation of the PTE plus the `INITIAL_FREEZE_THAW_EPOCHS` and just set an `ExtendedBoostThawStartBlockNumber`.
- The `ExtendedBoostThawStartBlockNumber` or `PrecipitatingEventBlockNumber` could be set via upgrade migration to be the same as the fallback value, although that would then not have the immediate 100% unlock ratio

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

All current ProviderBoost participants will be migrated to ProviderExtendedBoost during rollout.

Users who wish to opt-out must unstake before the upgrade.

## Example User actions for the Extended Boosting Program

1. Stake
    - Before the upgrade (to be migrated): `capacity.provider_boost(target, amount)`
    - After the upgrade: `capacity.extended_boost(target, amount)`
2. Claim Rewards
3. (Governance) PTE Happens, Time Passes
4. (If desired) Extended Thaw Period: User requests a partial unstake: `capacity.unstake(limited_amount)`
5. (If desired) Post Expiration: Unstake up to full amount: `capacity.unstake(full_amount)`
