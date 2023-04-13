# Capacity Staking Rewards

## Context and Scope:
The Frequency Transaction Payment system uses Capacity to pay for certain transactions on chain.  Accounts that wish to pay with Capacity must:
1. Have an [MSA](https://github.com/LibertyDSNP/frequency/blob/main/designdocs/accounts.md)
2. Be a [Provider](https://github.com/LibertyDSNP/frequency/blob/main/designdocs/provider_registration.md) (see also [Provider Permissions and Grants](https://github.com/LibertyDSNP/frequency/blob/main/designdocs/provider_permissions.md))
3. Stake a minimum amount of FRQCY token to receive [Capacity](https://github.com/LibertyDSNP/frequency/blob/main/designdocs/capacity.md).

There is also a business case for allowing any token holder to lock up its tokens in exchange for a reward - known as _staking_ - while also targeting a Provider to receive some Capacity.

The proposed feature is a design for staking FRQCY token in exchange for Capacity and/or FRQCY.
It is specific to the Frequency Substrate parachain.
It consists of enhancements to the capacity pallet, needed traits and their implementations, and needed runtime configuration.

## Glossary
1. **FRQCY**: the native token of the Frequency blockchain
1. **Capacity**: the non-transferrable utility token which can be used only to pay for certain Frequency transactions.
1. **Account**: a Frequency System Account controlled by a private key and addressed by a public key, having at least a minimum balance (currently 1 FRQCY).
1. **Stake** (verb): to lock some amount of a token against transfer for a period of time in exchange for some reward.
1. **Era**: the time period (TBD in blocks or Capacity Epochs) that Staking Rewards are based upon.
1. **Staking Reward**: a per-Era share of a staking reward pool of FRQCY tokens for a given staking account.
1. **Reward Pool**:  a fixed amount of FRQCY that can be minted for rewards each Era and distributed to stakers.

[//]: # (A short description of the landscape in which the new system is being built, what is actually being built.)
[//]: # (It may also say what is not being built, and any assumptions.)
[//]: # (Example: The proposed feature is a testing library. The context is: the library is for our chosen blockchain. The scope is: this is for a specific repository, so it's not meant to be reused. That means it won't be a separate package, and the code will be tailored for this repo. One might also say that the scope is also limited to developer testing, so it's not meant to be used in CI or a test environment such as a test blockchain network.)

## Problem Statement:
To build a self-sustaining Frequency network, a variety of economic incentives are needed.  One of these is the ability to stake FRQCY token in return for something.

[//]: # (The "why." A short summary of the issue&#40;s&#41; that this design solves. This doesn't have to be a technical problem, and it doesn't have to be a literal "problem." It could also be a necessary feature. "Developer unhappiness", "user experience needs improvement", are also problems.)

## Assumptions
* The exact formula for calculating rewards is provided in advance and used in the implementation of this design.
* The reward pool size is fixed until it is either set directly by governance or calculated from a value set by governance (which of these is TBD)
* Rewards are not prorated; they are calculated based on the staking account balance at the start of an Era.

### Example possible calculations
There are many possible calculations but here are some ideas about what factors could influence the rewards and how they could be incorporated.
* Reward pool is like simple interest. Network rewards are fixed at M / total staked token, where M is some number set by governance.

* Reward pool is linear, and is proportional to staked token and rewards decentralization of Providers. Example:
    1. Reward pool = (StakedToken + Providers\*1000) / 1000

* Reward pool is linear and rewards decentralization of stakers, decentralization of Providers.  Example:
    1. Reward pool = N\*StakedToken + (M\*Stakers + P\*Providers) / 1000, where N, M, and P are constants set by governance.

* Reward pool is a polynomial function:
    1. Reward pool = (N\*StakedToken + M\*Stakers^2 + P\*Providers^2) / sqrt(2), where N, M, and P are constants set by governance.

* There is no fixed reward pool, but rewards per FRQCY staked are calculated based on a logarithmic function:
    1. RewardPerFRQCY = N * e^-2\*(M\*Stakers + P\*Providers), where N, M, and P are constants set by governance.

## Goals
* Outline the architecture for the implementation of staking rewards for Capacity.
* Allow staking rewards parameters to be adjusted without a chain upgrade.
* Prevent rewards storage operations and calculations from excessively weighing down blocks.
* Disallow receiving staking rewards for token that is not staked for a full Era.
* Prevent unstaking "spam" which would destabilize the chain token economy and slow down block formation due to excessive database read/writes.
* Create a living design document that changes as compelling new findings and needs arise

## Non-goals
* Do not finalize names, functions, storage types and shape of storage and structs
* Do not determine the actual amount of rewards - either in Capacity or FRQCY - for staking
* Do not account for other economic incentives on the Frequency network, such as collator rewards.
* Cannot account for token price in any other currency.
* Do not change how paying for transactions with Capacity works

---
## Proposal:
Any Frequency account with a set minimum amount of FRQCY to may stake token to the network.
On staking, the staker designates a type of staking, plus a target Provider to receive Capacity.
There are two types of staking, **Maximized Rewards** and **Simple Rewards**.
In both types, the staker designates a target Provider who receives capacity upon staking.
The difference is:

* With **Maximized Capacity Staking**, the target Provider receives more Capacity than it would with Simple Rewards.
  The staker does not receive any token rewards.
* With **Rewards + Capacity Staking**, the target Provider shares rewards with the staker.
  The target Provider receives some Capacity, and the staker receives periodic rewards in FRQCY.

Whenever `StakingAccountDetails.total` changes, either by staking more or unstaking, rewards are calculated, minted and transferred immediately.  `StakingAccountDetails.total` is updated to be the current era.  Rewards are paid out from `last_rewarded_at` to `current_era() - 1` by applying the RewardsPoolParameters to each Era.

Whenever a Rewards Pool calculation is changed, it goes into effect at the start of the next Era. The new Rewards parameters are pushed to storage, and, assuming the history queue is full, the oldest one is removed.

A special condition of this is a blend of the above:  some staking accounts are large enough that if their balance changes significantly, it can affect the Reward Pool significantly for all staking accounts.  We choose to

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
}
```

### New storage items and related types

#### RewardsPoolHistory and RewardsPoolParameters
We store changes to the Rewards Pool calculations when any change occurs that affects the Rewards Pool greater than or equal to a hundredth of a percent (0.01%), rounded up, or when Rewards Pool Parameters are changed.

**Toy Example:**
Let's assume a simple reward pool calculation relates to total token staked and the number of stakers, multiplied by some constant:
```rust
RewardPoolTokenAmount = (TotalToken + 1000*NumStakers)
                        _____________________________
                                 100,000
```
Say total staked token is 100k FRQCY, there are 25 stakers, and Moby staker has staked 20k FRQCY.  The reward pool starts at 125k/100k  = 1.25 FRQCY to spread among 25 stakers based on their stake.

Moby account unstakes 10k FRQCY, which drops staked token by 10%.  The new reward pool is (90,000 + 25,000) / 100,000 = 1.15 FRQCY to spread among the stakers.  This causes the Reward Pool to go down by 12%, which is much larger than 0.01%), so a new entry is pushed into the RewardPool history.

```rust
/// A queue of the last `RewardsPoolHistoryMaxDepth` RewardsPoolParameters.
type RewardsPoolHistory<T>: BoundedVec<RewardsPoolParameters<T>, T::RewardsPoolHistoryMaxDepth>;
```

```rust
/// The parameters for a rewards pool that applied from `applied_at` StakingEra to the next time
/// the parameters changed.
pub struct RewardsPoolParameters<T: CapacityConfig> {
    /// the total of all FRQCY staked when these parameters were applied
    pub staking_total: BalanceOf<T>,
    /// the size of the rewards pool when these parameters were applied
    pub rewards_pool_total: BalanceOf<T>
    /// the first era these parameters were applied
    pub applied_at: T::StakingEra,
    /// the number of providers
    pub providers: uint32,
    /// the number of stakers
    pub stakers: uint32,
}
```

### Properties and Pallet Storage (DRAFT! some of these are just a guess!)
* `staking_type` (Maximized or Simple): An enum. A property on `StakingAccountDetails`
* `pub type RewardRate: StorageValue<_, uint16, ValueQuery>`: Stores the reward rate as hundredths of a percent; a reward rate of 1.25% would be stored as `125`.
* `pub type RewardFrequency<T: Config> StorageValue<_, <T: BlockNumber` _\[or EpochNumber\]_ `>, ValueQuery>`: Stores Reward frequency in number of epochs or blocks, **TBD**
* `pub type CapacityPriceMaximized<T: Config> StorageValue<_, BalanceOf<T>, ValueQuery>`:  Stores the price of 1 capacity in FRQCY, in the case of Maximized Staking.
* `pub type CapacityPriceSimple<T: Config> StorageValue<_, BalanceOf<T>, ValueQuery>`: Stores the price of 1 capacity in FRQCY, in the case of Simple/Rewards Staking.

### Capacity Pallet extrinsics
* `change_reward_rate(origin, rate)`: governance-only
* `change _reward_frequency(origin, period)`: governance-only
* `change_staking_type(origin, target, new_type)`
* `change_target(origin, old_target, new_target)`:

### Capacity Pallet helper functions
* `participation_rate`: function, calculates participation using some combination of Total amount staked, reward Pool Token Size, the number of providers, the provider capacity amount (total?), individual amount staked (for individual reward amount only)
* `current_staking_reward(account_id, target)`: function, calculates the current rewards that would be paid out if the StakingAccount holder unstaked at the current block.
* `pay_reward`: function,

### RPC
* `unclaimed_staking_reward()`: caller can query unclaimed staking reward.

### Other functions
* `change_staking_type`:  a function on StakingAccountDetails  (? maybe you have to unstake and restake to do this)

[//]: # (A high level overview, followed by a detailed description.)
[//]: # (This is where specific technology, such as language, frameworks, encryption algorithms, type of authentication, and APIs can be described.)
[//]: # (It can include diagrams such as a system context diagram, or a sequence diagram.)

---
## Benefits and Risks:
### Benefit: stabler message costs
Staking locks up token.  Locked token may not be immediately withdrawn; this dampens some level of speculation-driven volatility as well as that driven by opportunistic Capacity purchases.

### Benefit: improved engagement and expanded user base
A Provider may, for example, airdrop tokens to users who meet certain criteria, such as referrals or sharing links on other platforms.  Users with token may choose Simple Reward staking to generate Capacity for their Provider and also get token rewards.

### Benefit: improved economic sustainability
A staking reward system can improve /onboard/uptake/usage/...

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
If Frequency were to wait until a withdrawal to calculate rewards, it would make accounting extremely complicated; since account holders may adjust their stake up or down, the chain would have to keep a ledger of amounts for every staking reward period, and if the reward rates change, that makes it even more complicated.  Rewards may not be applied off-chain, since token balances and staking rewards are part of consensus, and must be validatable.


### 1. Providers simply purchase capacity without staking (locking token balance)
### 2. Accounts stake only for token and/or to be collators
### 3. Only Simple Rewards staking type
### 4. Rewards are accounted for periodically all together, at once.
this puts an extreme burden on one or more blocks due to storage updates.
### 5. At least some portion of rewards are accounted for every block; all rewards are updated for all stakers within the Era.
This effectively creates a constant overhead, but this approach causes much heavier blocks than the chosen solution.
### 6. Stakers for a given Provider could be "lazily" rewarded at the same time a Provider posts a new message at the beginning of an Epoch.  This presents a problem if a Provider does not post every Epoch, especially if the Rewards Era is less than or equal to an Epoch.  This will also put a lot of extra weight on blocks at the beginning of every Epoch.
### 7. Pay rewards out every time there is a change for a given token staking account, or a change in Rewards parameters.
Since the only thing that changes staking rewards is the `StakingAccountDetails.total`, unless the Rewards calculation changes, we don't need to do a sweep unless the staker changes their total, either through staking more or unstaking.  This minimizes the average block burden significantly, however, occasionally, when the Rewards calculation changes, block time will be slower for the next Era due to rewards payouts.

### Why can't Frequency use Substrate `staking` pallet?
The staking pallet is for rewarding node validators, and rewards must be claimed within `HISTORY_DEPTH` blocks before the record of them is purged.  Reward payouts for a given validator can be called by any account and the rewards go o the validator and are shared with its nominators. The staking pallet keeps track of what rewards have been claimed within that `HISTORY_DEPTH` number of blocks.

Since Capacity Rewards staking is for FRQCY token account holders, we should not require those holders to have to call an extrinsic regularly to receive rewards.

Secondly, we must plan for the number of Providers to dwarf the number of validators on the Polkadot Relay chain.  The Polkadot relay chain currently has validators in the hundreds.  Calculating the payouts for hundreds of items in RocksDB storage is a very different prospect than calculating rewards for thousands of Providers and potentially tens of millions (or more) of Rewards stakers.  It may be that this type of optimization is deferred, however, this design must not make it difficult to optimize.

[//]: # (discuss alternatives that were considered, and why they were rejected.)
[//]: # (Note when there are absolute requirements that the solution does not and can't meet. One example might be, it's a proprietary solution but we need something open source.)

## Sources:

[//]: # (sources of information that led to this design.)
