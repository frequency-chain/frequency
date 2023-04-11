# Capacity Staking Rewards

## Context and Scope:
The Frequency Transaction Payment system uses Capacity to pay for certain transactions on chain.  Accounts that wish to pay with FRQCY must:
1. Have an [MSA](https://github.com/LibertyDSNP/frequency/blob/main/designdocs/accounts.md)
2. Be a [Provider](https://github.com/LibertyDSNP/frequency/blob/main/designdocs/provider_registration.md) (see also [Provider Permissions and Grants](https://github.com/LibertyDSNP/frequency/blob/main/designdocs/provider_permissions.md)
3. Stake token to receive [Capacity](https://github.com/LibertyDSNP/frequency/blob/main/designdocs/capacity.md).

The proposed feature is a design for staking FRQCY token in exchange for Capacity and/or FRQCY.
It consists of enhancements to the capacity pallet, needed traits and their implementations, and needed runtime configuration.

[//]: # (A short description of the landscape in which the new system is being built, what is actually being built.)
[//]: # (It may also say what is not being built, and any assumptions.)
[//]: # (Example: The proposed feature is a testing library. The context is: the library is for our chosen blockchain. The scope is: this is for a specific repository, so it's not meant to be reused. That means it won't be a separate package, and the code will be tailored for this repo. One might also say that the scope is also limited to developer testing, so it's not meant to be used in CI or a test environment such as a test blockchain network.)

## Problem Statement:
To build a self-sustaining Frequency network, a variety of economic incentives are needed.  One of these is the ability to stake token in return for something.

[//]: # (The "why." A short summary of the issue&#40;s&#41; that this design solves. This doesn't have to be a technical problem, and it doesn't have to be a literal "problem." It could also be a necessary feature. "Developer unhappiness", "user experience needs improvement", are also problems.)

## Goals
* Outline the architecture for the implementation of staking rewards for Capacity.
* Allow staking rewards parameters to be adjusted without a chain upgrade.
* Design prevents rewards storage updates and calculations from excessive weighing down of blocks.

## Non-goals
* Do not determine the actual amount of rewards - either in Capacity or FRQCY - for staking.
* Do not account for other economic incentives on the Frequency network, such as collator rewards.
* Cannot account for token price in any other currency.
* Do not change how paying for transactions with Capacity works

[//]: # (what this solution is trying to do, and is also not trying to do, in concrete terms. Measurable goals are best.)
[//]: # (## Glossary &#40;optional&#41;:)
[//]: # (if you think inline links to concepts are too distracting, include a glossary section. This can be links, text or both.)

---
## Proposal:
Any Frequency account with a set minimum amount of FRQCY to may stake token to the network.
On staking, the staker designates a type of staking, plus a target Provider to receive Capacity.
There are two types of staking, **Maximized Rewards** and **Simple Rewards**.
In both types, the staker designates a target Provider who receives capacity upon staking.
The difference is:

* With **Maximized Rewards**, the target Provider receives more Capacity than it would with Simple Rewards.
  The staker does not receive any token rewards.
* With **Simple Rewards**, the target Provider shares rewards with the staker.
  The target Provider receives some Capacity, and the staker receives periodic rewards in FRQCY.

## Staking Token Rewards
If Frequency were to wait until a withdrawal to calculate rewards, it would make accounting extremely complicated; since account holders may adjust their stake up or down, the chain would have to keep a ledger of amounts for every staking reward period, and if the reward rates change, that makes it even more complicated.  Rewards may not be applied off-chain, since token balances and staking rewards are part of consensus, and must be validatable.

### Some options considered:
1. Rewards are accounted for periodically all together, at once; this puts an extreme burden on one or more blocks due to storage updates.
2. At least some portion of rewards are accounted for every block, however, all rewards must be updated for all stakers within the Era.
3. Stakers for a given Provider could be "lazily" rewarded at the same time a Provider posts a new message at the beginning of an Epoch.  This presents a problem if a Provider does not post every Epoch, especially if the Rewards Era is less than or equal to an Epoch.  This will also put a lot of extra weight on blocks at the beginning of every Epoch.

### Why can't Frequency use Substrate `staking` pallet?  [or can it?]
The staking pallet is for rewarding node validators, and rewards must be claimed within `HISTORY_DEPTH` blocks before the record of them is purged.  Reward payouts for a given validator can be called by any account and the rewards go o the validator and are shared with its nominators. The staking pallet keeps track of what rewards have been claimed within that `HISTORY_DEPTH` number of blocks.

Since Capacity Rewards staking is for FRQCY token account holders, we should not require those holders to have to call an extrinsic regularly to receive rewards.

Secondly, we must plan for the number of Providers to dwarf the number of validators on the Polkadot Relay chain.  The Polkadot relay chain currently has validators in the hundreds.  Calculating the payouts for hundreds of items in RocksDB storage is a very different prospect than calculating rewards for thousands of Providers and potentially tens of millions (or more) of Rewards stakers.  It may be that this type of optimization is deferred, however, this design must not make it difficult to optimize.

### Properties and Pallet Storage
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
* `current_staking_reward()`: caller can force the calculation of a staking reward.

### Other functions
* change_staking_type:  a function on StakingAccountDetails
*

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
### 1. Providers simply purchase capacity without staking (locking token balance)
### 2. Accounts stake only for token and/or to be collators
### 3. Only Simple Rewards staking type
### 4. ??

[//]: # (discuss alternatives that were considered, and why they were rejected.)
[//]: # (Note when there are absolute requirements that the solution does not and can't meet. One example might be, it's a proprietary solution but we need something open source.)

## Sources:

[//]: # (sources of information that led to this design.)
