# Capacity Staking Rewards Implementation
The proposed feature is a design for staking FRQCY token in exchange for Capacity and/or FRQCY.
It is specific to the Frequency Substrate parachain.
It consists of enhancements to the capacity pallet, needed traits and their implementations, and needed runtime configuration.

This does _not_ outline the economic model for Staking Rewards (also known as "Provider Boosting"); it describes the economic model as a black box, i.e. an interface.

## Context and Scope:
The Frequency Transaction Payment system uses Capacity to pay for certain transactions on chain.  Accounts that wish to pay with Capacity must:

1. Have an [MSA](https://github.com/LibertyDSNP/frequency/blob/main/designdocs/accounts.md)
2. Be a [Provider](https://github.com/LibertyDSNP/frequency/blob/main/designdocs/provider_registration.md) (see also [Provider Permissions and Grants](https://github.com/LibertyDSNP/frequency/blob/main/designdocs/provider_permissions.md))
3. Stake a minimum amount of FRQCY (on mainnet, UNIT on Rococo testnet) token to receive [Capacity](https://github.com/LibertyDSNP/frequency/blob/main/designdocs/capacity.md).

# Problem Statement
This document outlines how to implement the Staking for Rewards feature described in [Capacity Staking Rewards Economic Model (TBD)](TBD), without, at this time, regard to what the economic model actually is.

## Glossary
1. **FRQCY**: the native token of the Frequency blockchain
1. **Capacity**: the non-transferrable utility token which can be used only to pay for certain Frequency transactions.
1. **Account**: a Frequency System Account controlled by a private key and addressed by a public key, having at least a minimum balance (currently 0.01 FRQCY).
1. **Stake** (verb): to lock some amount of a token against transfer for a period of time in exchange for some reward.
1. **RewardEra**: the time period (TBD in blocks or Capacity Epochs) that Staking Rewards are based upon. RewardEra is to distinguish it easily from Substrate's staking pallet Era, or the index of said time period.
1. **Staking Reward**: a per-RewardEra share of a staking reward pool of FRQCY tokens for a given staking account.
1. **Reward Pool**:  a fixed amount of FRQCY that can be minted for rewards each RewardEra and distributed to stakers.
1. **StakingRewardsProvider**: a trait that encapsulates the economic model for staking rewards, providing functionality for calculating the reward pool and staking rewards.

## Staking Token Rewards

### StakingAccountDetails updates
New fields are added. The field `last_rewarded_at` is to keep track of the last time rewards were claimed for this Staking Account.
MaximumCapacity staking accounts MUST always have the value `None` for `last_rewarded_at`.  This should be the default value also.
`MaximumCapacity` is also the default value for `staking_type` and should map to 0.
Finally, `stake_change_unlocking`, a BoundedVec is added which tracks the chunks of when a staking account has changed targets for some amount of funds.
```rust
pub struct StakingAccountDetails {
    pub active: BalanceOf<T>,
    pub total: BalanceOf<T>,
    pub unlocking: BoundedVec<UnlockChunk<BalanceOf<T>, T::EpochNumber>, T::MaxUnlockingChunks>,
    /// The number of the last StakingEra that this account's rewards were claimed.
    pub last_rewards_claimed_at: Option<T::RewardEra>, // NEW  None means never rewarded, Some(RewardEra) means last rewarded RewardEra.
    /// What type of staking this account is doing
    pub staking_type: StakingType, // NEW
    /// staking amounts that have been retargeted are prevented from being retargeted again for the
    /// configured Thawing Period number of blocks.
    pub stake_change_unlocking: BoundedVec<UnlockChunk<BalanceOf<T>, T::RewardEra>, T::MaxUnlockingChunks>, // NEW
    /// total staked amounts for each past era, up to StakingRewardsPastErasMax eras.
    pub staking_history: BoundedVec<StakingHistory<BalanceOf<T>, T::RewardEra>, T::StakingRewardsPastErasMax>, // NEW
}

pub struct StakingHistory<Balance, RewardEra> {
    total_staked: Balance,
    reward_era: RewardEra,
} 
```

**Unstaking thaw period**
Changes the thaw period to begin at the first block of next RewardEra instead of immediately.

### Changes to extrinsics
```rust
pub fn stake(
    origin: OriginFor<T>,
    target: MessageSourceId,
    amount: BalanceOf<T>,
    staking_type: StakingType // NEW
) -> DispatchResult {
    /// NEW BEHAVIOR:
    // if the account is new, save the new staking type
    // if not new and staking type is different, Error::CannotChangeStakingType
}

pub fn unstake(
    origin: OriginFor<T>,
    target: MessageSourceId,
    requested_amount: BalanceOf<T>,
) -> DispatchResult {
    // NEW BEHAVIOR:
    // If StakingType is RewardsType
    //   If payout_eligible,
    //     check whether their last payout era is recent enough to pay out all rewards at once.
    //     if so, first pay out all rewards and then continue with rest of unstaking code as is
    //     if not, emit error "MustFirstClaimUnclaimedRewards", "UnclaimedRewardsOverTooManyEras" or something like that
    //   If not payout eligible,
    //     check whether the last payout era is the current one.
    //     if so, all rewards have been claimed, so continue with rest of unstaking code as is,
    //
    //     otherwise, they have too many unlocking chunks so they'll have to wait. - the unstaking code
    //     will catch this anyway and emit `MaxUnlockingChunksExceeded`
}
```
### NEW: StakingRewardsProvider - Economic Model trait
This one is most likely to change, however there are certain functions that will definitely be needed.
The struct and method for claiming rewards is probably going to change, but the rewards system will still need to know the `reward_pool_size` and the `staking_reward_total` for a given staker.

```rust
use std::hash::Hash;

pub struct StakingRewardClaim<T: Config> {
    /// How much is claimed, in token
    pub claimed_reward: Balance,
    /// The end state of the staking account if the operations are valid
    pub staking_account_end_state: StakingAccountDetails,
    /// The starting era for the claimed reward period, inclusive
    pub from_era: T::RewardEra,
    /// The ending era for the claimed reward period, inclusive
    pub to_era: T::RewardEra,
}

pub trait StakingRewardsProvider<T: Config> {

    /// Calculate the size of the reward pool for the given era, in token
    fn reward_pool_size() -> BalanceOf<T>;

    /// Return the total unclaimed reward in token for `account_id` for `fromEra` --> `toEra`, inclusive
    /// Errors:
    ///     - EraOutOfRange when fromEra or toEra are prior to the history retention limit, or greater than the current RewardEra.
    fn staking_reward_total(account_id: T::AccountId, fromEra: T::RewardEra, toEra: T::RewardEra);

    /// Validate a payout claim for `account_id`, using `proof` and the provided `payload` StakingRewardClaim.
    /// Returns whether the claim passes validation.  Accounts must first pass `payoutEligible` test.
    /// Errors: None
    fn validate_staking_reward_claim(account_id: T::AccountID, proof: Hash, payload: StakingRewardClaim<T>) -> bool;
}
```

### NEW: StakingType enum
```rust
pub enum StakingType {
    /// Staking account targets Providers for capacity only, no token reward
    MaximizedCapacity,
    /// Staking account targets Providers and splits reward between capacity to the Provider
    /// and token for the account holder
    Rewards,
}
```

### NEW: Config items
```rust
pub trait Config: frame_system::Config {
    // ...

    /// A period of `EraLength` blocks in which a Staking Pool applies and
    /// when Staking Rewards may be earned.
    type RewardEra:  Parameter
                + Member
                + MaybeSerializeDeserialize
                + MaybeDisplay
                + AtLeast32BitUnsigned
                + Default
                + Copy
                + sp_std::hash::Hash
                + MaxEncodedLen
                + TypeInfo;
    /// The number of blocks in a Staking RewardEra
    type EraLength: Get<u32>;
    /// The maximum number of eras over which one can claim rewards
    type StakingRewardsPastErasMax: Get<u32>;

    type RewardsProvider: StakingRewardsProvider;
};
```

### NEW: RewardPoolInfo
This is the necessary information about the reward pool for a given Reward Era and how it's stored.
```rust
pub struct RewardPoolInfo<Balance> {
    /// the total staked for rewards in the associated RewardEra
    pub total_staked_token: Balance,
    /// the reward pool for this era
    pub total_reward_pool: Balance,
    /// the remaining rewards balance to be claimed
    pub unclaimed_balance: Balance,
}

/// Reward Pool history
#[pallet::storage]
#[pallet::getter(fn get_reward_pool_for_era)]
pub type StakingRewardPool<T: Config> = <StorageMap<_, Twox64Concat, RewardEra, RewardPoolInfo<T>;
```

### NEW: CurrentEra, RewardEraInfo
Incremented, like CurrentEpoch, tracks the current RewardEra number and the block when it started.
```rust
#[pallet::storage]
#[pallet::whitelist_storage]
#[pallet::getter(fn get_current_era)]
/// Similar to CurrentEpoch
pub type CurrentEraInfo<T:Config> = StorageValue<_, T::RewardEraInfo, ValueQuery>;

pub struct RewardEraInfo<RewardEra, BlockNumber> {
    /// the index of this era
    pub current_era: RewardEra,
    /// the starting block of this era
    pub era_start: BlockNumber,
}
```

### NEW: Error enums
```rust
pub enum Error<T> {
    /// ...
    /// Staker tried to change StakingType on an existing account
    CannotChangeStakingType,
    /// The Era specified is too far in the past or is in the future
    EraOutOfRange,
    /// Rewards were already paid out for the specified Era range
    IneligibleForPayoutInEraRange,
}
```

### NEW Extrinsics
1. *claim_staking_reward*, first version
    a. `claim_staking_reward(origin,proof,payload)`
    ```rust
    /// TBD whether this is the form for claiming rewards.  This could be the form if calculations are
    /// done off chain and submitted for validation.
    /// Validates the reward claim. If validated, mints token and transfers to Origin.
    /// Errors:
    ///     - NotAStakingAccount:  if Origin does not own the StakingRewardDetails in the claim.
    ///     - StakingRewardClaimInvalid:  if validation of calculation fails
    ///     - IneligibleForPayoutInEraRange:  if rewards were already paid out in the provided RewardEra range
    ///     - EraOutOfRange: if one or both of the StakingRewardClaim eras are invalid
    /// `proof` - the Merkle proof for the reward claim
    #[pallet::call_index(n)]
    pub fn claim_staking_reward(
        origin: OriginFor<T>,
        proof: Hash,
        payload: StakingRewardClaim<T>
    );
    ```
    b. *claim_staking_reward*, alternate version
    ```rust
    /// An alternative, depending on staking reward economic model. This could be the form if calculations are done on chain.
    /// from_era: if None, since last_reward_claimed_at
    /// to_era: if None, to CurrentEra - 1
    ///  Errors:
    ///     - NotAStakingAccount:  if Origin does not own the StakingRewardDetails in the claim.
    ///     - IneligibleForPayoutInEraRange:  if rewards were already paid out in the provided RewardEra range
    ///     - EraOutOfRange: if one or both of the eras specified are invalid
    #[pallet::call_index(n)]
    pub fn claim_staking_reward(
        origin: OriginFor<T>,
        from_era: Option<T::RewardEra>,
        to_era: Option<T::RewardEra>
    );
    ```
    Both emit events `StakingRewardClaimed` with the parameters of the extrinsic.

2. **change_staking_target(origin, from, to, amount)**
Changes a staking account detail's target MSA Id to a new one by `amount`
Rules for this are similar to unstaking; if `amount` would leave less than the minimum staking  amount for the `from` target, the entire amount is retargeted.
No more than T::MaxUnlockingChunks staking amounts may be retargeted within this Thawing Period.
Each call creates one chunk.  Emits a `StakingTargetChanged` event with the parameters of the extrinsic.

```rust
/// Errors:
///    - MaxUnlockingChunksExceeded if 'from' target staking amount is still thawing in the staking unlock chunks (either type)
///    - StakerTargetRelationshipNotFound` if `from` is not a staking target for Origin. This also covers when account's MSA is not staking anything at all or account has no MSA
///    - StakingAmountBelowMinimum if amount to retarget is below the minimum staking amount.
///    - InsufficientStakingBalance if amount to retarget exceeds what the staker has targeted to the `from` MSA Id.
///    - InvalidTarget if `to` is not a Registered Provider.
#[pallet:call_index(n+1)] // n = current call index in the pallet
pub fn change_staking_target(
    origin: OriginFor<T>,
    from: MessageSourceId,
    to: MessageSourceId,
    amount: BalanceOf<T>
);
```

### NEW:  Capacity pallet helper function
```rust
/// Return whether `account_id` can claim a reward. Staking accounts may not claim a reward more than once
/// per RewardEra, may not claim rewards before a complete RewardEra has been staked, and may not claim more rewards past
/// the number of `MaxUnlockingChunks`.
/// Errors:
///     NotAStakingAccount if account_id has no StakingAccountDetails in storage.
fn payout_eligible(account_id: AccountIdOf<T>) -> bool;
```

### NEW RPCS
There are no custom RPCs for the Capacity pallet, so that work will need to be done first.
```rust
pub struct UnclaimedRewardInfo {
    /// The Reward Era for which this reward was earned
    reward_era: RewardEra,
    /// An ISO8701 string, UTC, estimated using current block time, and the number of blocks between
    /// the current block and the block when this era's RewardPoolInfo would be removed from StakingRewardPool history 
    expires_at: string,
    /// The amount staked in this era
    staked_amount: BalanceOf<T>,
    /// The amount in token of the reward (only if it can be calculated using only on chain data)
    earned_amount: BalanceOf<T>
}

/// Check what unclaimed rewards origin has and how long they have left to claim them
/// If no unclaimed rewards, returns empty list.
fn check_for_unclaimed_rewards(origin: OriginFor<T>) -> Vec<UnclaimedRewardInfo>;
```
