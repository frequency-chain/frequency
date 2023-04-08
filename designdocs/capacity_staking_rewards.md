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
* Allow staking rewards to be adjusted without a chain upgrade.

## Non-goals
* This does not determine the actual amount of rewards - either in Capacity or FRQCY - for staking.
* This does not account for other economic incentives on the Frequency network, such as collator rewards.
* This design does not (and cannot) account for token price in any other currency.
* It does not change how paying for transactions with Capacity works

[//]: # (what this solution is trying to do, and is also not trying to do, in concrete terms. Measurable goals are best.)
[//]: # (## Glossary &#40;optional&#41;:)
[//]: # (if you think inline links to concepts are too distracting, include a glossary section. This can be links, text or both.)

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


[//]: # (A high level overview, followed by a detailed description.)
[//]: # (This is where specific technology, such as language, frameworks, encryption algorithms, type of authentication, and APIs can be described.)
[//]: # (It can include diagrams such as a system context diagram, or a sequence diagram.)

## Benefits and Risks:
### Benefit: stabilizing message costs
Staking locks up token.  Locked token may not be immediately withdrawn; this dampens some level of speculation-driven volatility as well as that driven by opportunistic Capacity purchases.

### Benefit: engage and expand user base
A Provider may, for example, airdrop tokens to users who meet certain criteria, such as referrals or sharing links on other platforms.  Users with token may choose Simple Reward staking to generate Capacity for their Provider and also get token rewards.

### Benefit: improved economic sustainability
A staking reward system can improve /onboard/uptake/usage/...

### Risk: Faulty reward calculations:
* Maximized Stake for capacity is not cheaper per txn than pay-as-you-go with token
* Maximized Stake for capacity pays better than staking to be a collator

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
