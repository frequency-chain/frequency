# Provider Boosting Economic Model

This document outlines the economic model to be used for:
1. determining the token value of the Reward Pool for a given Era
2. how to calculate rewards for an individual participant in the Provider Boost program
3. when rewards are calculated
4. when rewards are paid out
5. where these calculations are performed

## Context and Scope:
The Frequency Transaction Payment system uses Capacity to pay for a limited number of specific transactions on chain.  Accounts that wish to pay for transactions with Capacity must:
1. Have an [MSA](https://github.com/LibertyDSNP/frequency/blob/main/designdocs/accounts.md)
2. Be a [Provider](https://github.com/LibertyDSNP/frequency/blob/main/designdocs/provider_registration.md) (see also [Provider Permissions and Grants](https://github.com/LibertyDSNP/frequency/blob/main/designdocs/provider_permissions.md))
3. Lock up a minimum amount of FRQCY token to receive [Capacity](https://github.com/LibertyDSNP/frequency/blob/main/designdocs/capacity.md).

There is also a business case for allowing any token holder to lock up its tokens in exchange for a reward - while also targeting a Provider to receive some Capacity.


## Problem Statement:
A system consisting only of providers and coinless users who delegate to providers will tend toward centralization.
To build a self-sustaining Frequency network where control is decentralized, a variety of economic solutions are needed.  One of these is the ability to lock up FRQCY token in return for something; this creates an incentive for participation and involvement with the Frequency chain fundamentals and governance.

How is that so?  Capacity is how Frequency intends Providers to pay for the vast majority of their on-chain messages. In the proposed system, when Providers receive Capacity when end users lock up some FRQCY, they would lose Capacity if those end users unlock. If a Provider's Capacity from Provider Boosting is significant, this gives Provider Boosters some power over their targeted Providers. If a Provider is utilizing all or nearly all their Capacity almost every Epoch -- which they should do if trying to be economical --  then even a small percentage of lost Capacity will literally cost them to replace it.  This gives those end-users relying upon - and Boosting - their Providers the ability to exercise direct market power they did not previously have.

End users with Message Source Accounts (MSAs) may receive FRQCY from different sources. Providers may offer airdrops in return for such bringing in new users or sharing links on other platforms, then encourage their users to participate in Provider Boosting.  Rewards could potentially be exchanged for non-transferable, in-app-only benefits such as premium features, special emoji, avatar customization, and the like, similarly to platforms such as [Steam](https://store.steampowered.com).  

## Assumptions
* The exact formula for calculating rewards is determined in advance and used in the implementation of this design.

## Economic Model
"Economic model" means the formulas and inputs used to manage Provider Boost rewards to achieve the goals of decentralization, economic stabiilty, and sustainability.

## Goals
To specify the following:
* In words or pseudo-code how the reward pool and individual rewards are calculated
* How and when rewards are minted and transferred
* What to do with leftover and/or unclaimed funds set aside for Provider Boost Rewards.
* Limitations on receiving rewards and reward amounts

## Non-Goals
This document does not:
* specify implementation details or naming in code.
* specify reward amounts for all time; values and methods used for calculating rewards should be expected to change to meet economic goals of the Frequency Blockchain and any legal requirements.

## Proposal:
### Inputs to Provider Boost Reward Calculation
* R<sub>era</sub> is a predetermined amount of FRQCY available each Boost Era for Rewards
* L<sub>u</sub> is the amount an MSA holder has locked for Provider Boost Era <i>e</i>
* L<sub>T</sub> is the total that all MSA holders have locked for Provider Boost Era <i>e</i>
* P<sub>max</sub> is the maximum percentage of a Provider-Boosted amount that can be paid out in Era <i>e</i>

### Formula
The Provider Boost reward in FRQCY tokens for a given Era <i>e</i> is

R = <i>min</i>(R<sub>era</sub>*L<sub>u</sub>/L<sub>T</sub>, L<sub>u</sub>*P<sub>max</sub>)

Put into words, if the pool of Rewards per Era is R<sub>era</sub> FRQCY, then the Reward amount in FRQCY earned by a given Provider Booster will be proportional to how much they've locked for Provider Boosting out of the total, OR P<sub>max</sub> times the amount locked, whichever is less.

Put another way, there is a fixed number of tokens to be rewarded each Era (R<sub>era</sub>), split up according to each Provider Boost account holder's percentage of the locked total.  However, the reward return each Era for every individual account (P<sub>max</sub>) is capped at some rate, for example, 10%.

### Examples:
Given the following values:
* R<sub>era</sub> = 2 Million FRQCY
* R<sub>max</sub> is 10%

1. Ang has locked 100 FRQCY (L<sub>u</sub>) for Provider Boosting. The total locked by everyone, L<sub>T</sub> for era <i>e</i>,  is 10 Million FRQCY. The left side of the minimum is `2e6 * 100 / 10.0e6 = 100/5 = 20` (that is, 20% of what Ang has locked).   The right side is `100 * 10% = 10`.  Since 10 is less than 20, the reward amount is 10 FRQCY.
2. Bey has locked 1000 FRQCY  (L<sub>u</sub>)  for Provider Boosting.  The total locked by everyone, L<sub>T</sub> for era <i>e</i>,  is 50 Million FRQCY.   The left side of the minimum s `2e6 * 1000 / 50.0e6 = 1000/25 = 40` (that is, 4% of what Bey has locked).  The right side is `1000 * 10% = 100`.  Since 40 is less than 100, Bey's Provider Boost reward is 40 FRQCY.

## Rewards are not issued for a partial Era
Rewards are not prorated; they are calculated only for balances held for an entire Era.  For example, if an amount is locked at the end of Era 100, and unlocked in Era 101, _no_ Reward will be issued. If an amount is locked in Era 100, and unlocked in Era 102, a Provider Boost Reward is available _only_ for Era 101.

## Claiming Rewards
* Provider Boost Rewards are not minted until they are explicitly <i>claimed</i> by the Provider Boost account holder, by calling a non-free extrinsic.
* Rewards must be claimed within a certain number of Provider Boost Eras.
* When claimed, all available, unexpired Rewards for each previous Era are minted and transferred to the same account that locked them. 
* **Is there a cap on how much can be claimed at once?**
