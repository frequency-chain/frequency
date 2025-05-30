# 📄 Token Locking with Delayed Thaw and Boosting Rewards

## 📚 Context and Scope

This design document introduces a new `pallet-delayed_thaw` to the Frequency runtime. The primary focus is on defining a
mechanism for locking tokens in a way that is both time-sensitive and event-driven, with immutably defined thawing
behavior. While this design does not aim to change the Provider Boosting system itself, it leverages that mechanism to
provide users with an incentive to commit tokens to long-term locks. The `capacity` pallet's Provider Boosting system
serves as the vehicle for rewarding these users, with minimal changes required to accept staked tokens that are
simultaneously locked.

## ❗ Problem Statement
Business needs dictate that users be able to lock tokens subject to a Precipitating Tokenomic Event (PTE),
following which, tokens would unlock according to pre-set rules. The specifics of the locking/unlocking
scheme is not supported by the current `time-release` pallet.

Additionally:
- While the announcement of the PTE would be a Governance action, the subsequent ability to unlock
portions of the locked tokens over time should not be modifiable, even by Governance
- Tokens must be locked prior to the PTE as a condition of the PTE; however, if Governance fails
to act (or the PTE does not happen) by a certain date, there should be a failsafe whereby any
tokens so locked would automatically be able to be unlocked.
- Due to the exceesive lock period and delayed thaw, there should be some incentive mechanism for
users to participate in the extended locking scheme.
- 
## 🎯 Goals and Non-Goals

### Goals:

- Introduce a new pallet, `pallet-delayed_thaw`, to manage event-driven and schedule-based token unlocking.
- Enable tokens locked by `pallet-delayed_thaw` to participate in Provider Boosting.
- Provide separate reward behavior for locked tokens using the `RewardsProvider` trait.
- Ensure immutability of thawing logic while allowing governance to trigger the precipitating event.

### Non-Goals:

- This proposal does not change or refactor the general mechanics of Provider Boosting.
- It does not provide a user interface or wallet-level integration.
- It does not address consolidation or configuration of Provider Boosting parameters.

## ✨ Summary

This document proposes an enhancement to the `Provider Boosting` feature in the `capacity` pallet and the introduction
of a new `delayed_thaw` pallet to handle token locking logic. The objective is to support tokens locked with specific
thawing behavior, with such tokens participating in Provider Boosting at a distinct reward rate.

## 📂 `pallet-delayed_thaw` - Token Locking Mechanism

This new pallet manages the locking and gradual thawing of tokens. It does not handle reward logic but contributes to
reward calculation via a custom implementation of the `RewardsProvider` trait.

### Storage and Constants:

```rust
#[pallet::storage]
pub type PrecipitatingEventBlockNumber<T: Config> = StorageValue<_, T::BlockNumber, OptionQuery>;

pub const FAILSAFE_UNLOCK_BLOCK_NUMBER: BlockNumber = < some constant>;
```

- `PrecipitatingEventBlockNumber`: Set by governance to signal a precipitating event.
- `FAILSAFE_UNLOCK_BLOCK_NUMBER`: Ensures unlock safety if governance fails to act.

### Thaw Parameters:

```rust
pub const THAW_ERA_LENGTH: BlockNumber = < some constant>;
pub const INITIAL_FREEZE_THAW_ERAS: u32 = < some constant>;
pub const UNLOCK_THAW_ERAS: u32 = < some constant>;
```

**Unlock formula per era:**

```text
If current_era < INITIAL_FREEZE_THAW_ERAS:
    unlock_ratio = 0
Else:
    thaw_era = current_era - INITIAL_FREEZE_THAW_ERAS
    unlock_ratio = 1 / (UNLOCK_THAW_ERAS - min(thaw_era, UNLOCK_THAW_ERAS) + 1)
```

### Open Design Question:

> How should we enforce immutability of thaw parameters?

Options:

- **Hard-code into the pallet**: Ensures absolute immutability.
    - Chain code upgrade would be required to modify thaw parameters
- **Store with each lock**
    - Thaw modification would require both a chain code update and storage migration

**→ Feedback requested from the blockchain team.**

## 📈 Rewards Integration

### Interaction with Token Locks

The `pallet-delayed_thaw` will define a new `LockReason` for its locking operations. Since the balances pallet supports
multiple simultaneous locks under different `LockReason`s, tokens locked by `pallet-delayed_thaw` can still be
independently used for Provider Boosting. This is similar to how tokens locked for governance voting can also be reused
elsewhere in the runtime, provided the logic permits it.

To support this, the `capacity` pallet may require a minor change to its internal staking logic to accept tokens locked
under the `delayed_thaw` `LockReason` as valid staking collateral. This ensures seamless participation in boosting
rewards while maintaining lock-specific thawing behavior.

The rewards system remains in `pallet-capacity`. One change:

- `pallet-delayed_thaw` will implement the `RewardsProvider` trait.
- Allows for differential reward rate logic for locked tokens.

## 🔐 Governance Integration

- Governance can set `PrecipitatingEventBlockNumber`.
- No governance control over thaw parameters.

## 🔄 Migration Strategy

- Existing boost types and reward parameters remain unchanged.
- Integration of `pallet-delayed_thaw` adds new locked-stake source for boosting without altering core logic.
