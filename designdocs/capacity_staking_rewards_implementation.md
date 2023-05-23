# Capacity Staking Rewards Implementation
The proposed feature is a design for staking FRQCY token in exchange for Capacity and/or FRQCY.
It is specific to the Frequency Substrate parachain.
It consists of enhancements to the capacity pallet, needed traits and their implementations, and needed runtime configuration.

This does _not_ outline the Staking Rewards economic model; it describes the economic model as a black box, i.e. an interface.

## Context and Scope:
The Frequency Transaction Payment system uses Capacity to pay for certain transactions on chain.  Accounts that wish to pay with Capacity must:

1. Have an [MSA](https://github.com/LibertyDSNP/frequency/blob/main/designdocs/accounts.md)
2. Be a [Provider](https://github.com/LibertyDSNP/frequency/blob/main/designdocs/provider_registration.md) (see also [Provider Permissions and Grants](https://github.com/LibertyDSNP/frequency/blob/main/designdocs/provider_permissions.md))
3. Stake a minimum amount of FRQCY token to receive [Capacity](https://github.com/LibertyDSNP/frequency/blob/main/designdocs/capacity.md).

# Problem Statement
This document outlines how to implement the economic model described in [Capacity Staking Rewards Economic Model](TBD), specifically:
1. determining the token value of the Reward Pool for a given Era
2. how to calculate rewards for an individual staker
3. when rewards are calculated
4. when rewards are paid out
5. where these calculations are performed, and
6. Any other required changes to Capacity Staking as currently implemented

## Glossary
1. **FRQCY**: the native token of the Frequency blockchain
1. **Capacity**: the non-transferrable utility token which can be used only to pay for certain Frequency transactions.
1. **Account**: a Frequency System Account controlled by a private key and addressed by a public key, having at least a minimum balance (currently 0.01 FRQCY).
1. **Stake** (verb): to lock some amount of a token against transfer for a period of time in exchange for some reward.
1. **Era**: the time period (TBD in blocks or Capacity Epochs) that Staking Rewards are based upon.
1. **Staking Reward**: a per-Era share of a staking reward pool of FRQCY tokens for a given staking account.
1. **Reward Pool**:  a fixed amount of FRQCY that can be minted for rewards each Era and distributed to stakers.

## Staking Token Rewards

### StakingAccountDetails updates
We add a new field, `last_rewarded_at`, to keep track of the last time rewards were claimed for this Staking Account.
```rust
pub struct StakingAccountDetails {
    pub active: BalanceOf<T>,
    pub total: BalanceOf<T>,
    pub unlocking: BoundedVec<UnlockChunk<BalanceOf<T>, T::EpochNumber>, T::MaxUnlockingChunks>,
    /// The number of the last StakingEra that this account's rewards were claimed.
    pub last_rewarded_at: T::StakingEra, // NEW
    pub staking_type: StakingType, // NEW
}
```

**Unstaking thaw period**
We change the thaw period to begin at the first block of next Epoch instead of immediately.

### StakingRewardsProvider - Economic Model Interface

```rust
use std::hash::Hash;

pub struct StakingRewardClaim<T: Config> {
    pub claimedReward: BalanceOf<T>,
    pub newStakingAccountState: StakingAccountDetails<T: Config>
}

pub type StakingRewardsProvider<T: Config> {
type BalanceOf<T>;
type AccountIdOf<T>;
type EraOf<T>;

pub fn rewardPoolSize(era: EraOf<T>) -> BalanceOf<T>;

pub fn stakingRewardFor(accountId: AccountIdOf<T>, era: EraOf<T>) -> BalanceOf<T>;

pub fn stakingRewardTotal(accountId: AccountIdOf<T>, fromEra: EraOf<T>, toEra: EraOf<T>);

pub fn validateStakingRewardClaim(accountId: AccountIdOf<T>, proof: Hash, payload: StakingRewardClaim<T>) -> bool;

pub fn payoutEligible(accountId: AccountIdOf<T>) -> bool;
}
```


### New storage items and related types
```rust
#[pallet::storage]
#[pallet::getter(fn get_reward_pool_for)]
pub type StakingRewardPool<T: Config> =
    <StorageMap<_, Twox64Concat, T::Era, BalanceOf<T>>;
```

### New extrinsics
1. **claimStakingReward(origin,proof,payload)**
```rust
/// validates the reward claim
/// if validated, mints token and transfers to Origin.
/// Errors:
///     - if Origin does not own the StakingRewardDetails in the claim.
///     - if validation of calculation fails
///     - if rewards were already paid out in the current Era
#[pallet::call_index(n)]
pub fn claimStakingReward(
    origin: OriginFor<T>,
    proof: Hash,
    payload: StakingRewardClaim<T>
);
```

2. **change_staking_target(origin, from, to, amount)**
```rust
/// change a staking account detail's target MSA Id to a new one.
/// If amount is specified, that amount up to the total staking amount is retargeted,
/// otherwise ALL of the total staking amount for 'from' is changed to the new target MSA Id.
/// Errors:
///    -  if 'from' has no funds targeted in the staking account
///    - if 'to' MSA Id does not exist or is not a Provider
///    - if Origin does not have a staking account.
#[pallet:call_index(n+1)]
pub fn change_staking_target(
    origin: OriginFor<T>,
    from: MessageSourceId,
    to: MessageSourceId,
    amount: Option<BalanceOf<T>>
);
```

### RPC

## Benefits and Risks:
### Benefit: stabler message costs
Staking locks up token.  Locked token may not be immediately withdrawn; this dampens some level of speculation-driven volatility as well as that driven by opportunistic Capacity purchases.

### Benefit: improved engagement and expanded user base
A Provider may, for example, airdrop tokens to users who meet certain criteria, such as referrals or sharing links on other platforms.  Users with token may choose Reward staking to generate Capacity for their Provider and also get token rewards.

### Benefit: improved economic sustainability
A staking reward system can improve /onboard/uptake/usage/...

### Risk: staking system incentivizes gaming
If the staking system is too complicated, it can breed exploits and encourage gaming the systme.
If the staking system does not make it worthwhile to stake for rewards, the goal of decentralization is not achieved, and/or it can encourage gaming the system

# Risk: staking system penalizes whale Providers
If the system is unreliable, unstable or not economical enough for large Providers -- _particularly_ compared to alternatives -- the goal of decentralization may be achieved in some sense, but at the expense of widespread adoption.

### Risk: Faulty reward calculations:
* Maximized Stake for capacity is not cheaper per txn than pay-as-you-go with token
* Maximized Stake for capacity pays better than staking to be a collator

### Arguments in favor of storage values for reward rate and capacity price
* transparency:  it's more transparent than a Config, which could be changed only by an upgrade. This is because changes to Config values can be easily overlooked if they are buried in a large upgrade. Making them be subject to governance approval puts the change on chain, making it more subject to review.
* stabler: reward rates and capacity prices would have an automatic upper limit to how frequently they could change.

### Arguments against
* risk to network sustainability:  it's possible that proposed changes which would actually be necessary to Frequency's economic stability and sustainability may be rejected by token voters. This is mitigated particularly at the start of Frequency's operation given the token distribution, and also with the voting power and permissions of Frequency and Technical Councils.

#### Mitigation:
Adjust reward amounts. This is why the reward amounts need to be adjustable.

[//]: # (the reasons why this solution was chosen, and the risks this solution poses.)
[//]: # (For example, the solution may be very simple, but there could performance bottlenecks above a certain threshold.)
[//]: # (Another: the solution is well known and widely used, but it's not a perfect fit and requires complicated changes in one area.)

## Alternatives and Rationale:

### Why can't Frequency use Substrate `staking` pallet?
The staking pallet is for rewarding node validators, and rewards must be claimed within `HISTORY_DEPTH` blocks before the record of them is purged.  Reward payouts for a given validator can be called by any account and the rewards go o the validator and are shared with its nominators. The staking pallet keeps track of what rewards have been claimed within that `HISTORY_DEPTH` number of blocks.

Since Capacity Rewards staking is for FRQCY token account holders, we should not require those holders to have to call an extrinsic regularly to receive rewards.

Secondly, we must plan for the number of Providers to dwarf the number of validators on the Polkadot Relay chain.  The Polkadot relay chain currently has validators in the hundreds.  Calculating the payouts for hundreds of items in RocksDB storage is a very different prospect than calculating rewards for thousands of Providers and potentially tens of millions (or more) of Rewards stakers.  It may be that this type of optimization is deferred, however, this design must not make it difficult to optimize.

[//]: # (discuss alternatives that were considered, and why they were rejected.)
[//]: # (Note when there are absolute requirements that the solution does not and can't meet. One example might be, it's a proprietary solution but we need something open source.)

## Sources:

[//]: # (sources of information that led to this design.)
