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
To build a self-sustaining Frequency network where control is decentralized, a variety of economic solutions are needed.  One of these is the ability to lock up FRQCY token in return for something; this creates an incentive for participation and involvement with the chain fundamentals and governance.

How is that so?  Capacity is how Frequency intends Providers to pay for the vast majority of their on-chain messages. If Providers receive some capacity when end users lock up some token, then they lose Capacity if those end users unlock. If a Provider's Capacity from Provider Boosting is significant, this gives Provider Boosters some power over their targeted Providers. If a Provider is utilizing all or nearly all their Capacity almost every Epoch -- which they should do if trying to be economical --  then even a small percentage of lost Capacity will literally cost them to replace it.

FRQCY could potentially be airdropped by various entities to encourage onboarding. Providers may offer airdrops in return for such bringing in new users or sharing links on other platforms, then encourage their users to participate in Provider Boosting.  Rewards could potentially be exchanged for non-transferable, in-app-only benefits such as premium features, special emoji, avatar customization, and the like, similarly to platforms such as [Steam](https://store.steampowered.com).  

## Assumptions
* The exact formula for calculating rewards is determined in advance and used in the implementation of this design.
* Rewards are not prorated; they are calculated based on the Provider Boosted account balance at the start of each Era.

## Economic Model
By "economic model" we mean the formulas used to manage Provider Boost rewards to achieve the goals of decentralization, economic stabiilty, and sustainability.

## Goals
To specify the following:
1. In words or pseudo-code how the reward pool and individual rewards are calculated
2. When reward amounts are minted
3. How much Capacity is generated for the targeted Provider for a Provider Boost.
4. What to do with any left over funds set aside for Provider Boost Rewards.

## Non-Goals
1. Do not specify implementation details
2. Do not specify final naming

## Proposal:
### Inputs to Provider Boost Reward Calculation
- Number of complete Eras of Provider Boosting
- System-wide Totals Provider-Boosted each era
- Individual totals Provider-Boosted each era
- Rewards Cap amount
- Era reward percentage

