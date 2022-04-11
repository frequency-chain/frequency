# Delegations
This document describes the permissioned delegation of actions, largely, but not limited to, account creation and announcing messages by the owner of an AccountID on chain on behalf of the owner of another AccountID.

## Table of Contents
* [Context and Scope](https://github.com/LibertyDSNP/meta#installation)
* [Problem Statement](https://github.com/LibertyDSNP/meta#dependenciesrequirements)
* [Goals and Non-Goals](https://github.com/LibertyDSNP/meta#configuration)
* [Proposal](https://github.com/LibertyDSNP/meta#examples)
* [Benefits and Risks](https://github.com/LibertyDSNP/meta#roadmap)
* [Alternatives and Rationale](https://github.com/LibertyDSNP/meta#support)
* [Additional Resources](https://github.com/LibertyDSNP/meta#contributing)
* [Glossary](https://github.com/LibertyDSNP/meta#overview)

## Context and Scope
This document describes how a delegation is created and validated on chain.
Delegations can be used to perform tasks on behalf of another AccountId.
Some examples of delegated actions and delegated permissions are given.
It's expected that the actions and permissions that are implemented for delegation will evolve as needed.

## Problem Statement
The primary motivation for delegation is to support End Users of the DSNP platform, however, it is expected that delegation will be used in other ways.

Market research makes it clear that End Users are extremely reluctant to pay to use applications, particularly social networks.
This means there needs to be some way to onboard End Users and relay their activity through the DSNP platform without charging them.
On Ethereum and now on MRC, the use of authorized Delegates enables the creation of End User Accounts as well as processing and storing user messages and other data for the End Users, paid for by a Provider, who can recoup these costs by other means (outside the scope of this Design Document).
The vast majority of this activity will not reside on chain, however, MRC needs to be able to coordinate the exchange of data.

## Goals and Non-Goals
Delegation, roughly speaking, must allow all Create, Read, Update and Delete (CRUD) operations by a Delegating Account to fulfill the purpose of giving Account holders proper authority over their Delegates.
Put another way, delegation must have the following properties:
* **Authorizable** - delegations must be able to be authorized with specific permissions by Accounts.
* **Verifiable** - verifiability allows others to check that Providers are doing things only when authorized and only what they are authorized to do.
* **Transparent** - in order to verify Delegates, the delegations must be readable by anyone, in order to maximize opportunities to police Provider actions.
* **Changeable** - a Delegate's permissions must be able to be changed by the Delegator to give Account holders control over what tasks are permitted to the Delegate.
* **Revocable** - a Delegate's permissions must be able to be revoked by the Delegator to give Account holders the ability to withdraw permissions completely from the Delegate.

## Proposal
The proposed solution is to give End Users the ability to transparently authorize Delegates on chain and control what activities are delegated.

## Benefits and Risks
As stated earlier, one of the primary intended benefits of delegation is to allow feeless account creation and announcing.

## Alternatives and Rationale
### End User pays for existential deposit
We briefly discussed the possibility of requiring a small token deposit to create their account.
This was discarded because:
1. As mentioned above, people don't expect and won't pay to use social media.
2. The other problem would be onboarding people; even if they did want to pay even a small amount, getting people access to a token is tremendously difficult at this time, requiring unacceptable tradeoffs.
3. We would be unable to even serve a large number of people who are unbanked or don't have access to crypto trading platforms.

### dApp Developer pays for existential deposit
One alternative to allow for account creation at no cost to the End User was to have the dApp developer Account to send an existential deposit to the account to create it.
We decided against this option for a number of reasons.
1. It could create a potential for abuse and token loss by those creating numerous fake accounts and then removing the dApp AccountID as a delegate.
2. We have the ability not to require an existential deposit, and felt this to be a better option in this particular case.

### End user pays to announce
An alternative for delegating announcement capabilities was to have each End User pay for their own announcing.
We decided against this because
1. It's hard to persuade the average person to spend money to use social media.
2. This would require people to have to sign transactions every time they made any updates -- all posts, all reactions, all replies, all profile changes, all follows/unfollows, etc. It would be burdensome, create a lot of friction and be an overall annoying experience.
3. The delegation solution can be opt-in, so that if an End User did not want to trust a delegate, they could still pay for all of their announcing if they want to assume the cost of running a node and paying directly.

### Permissioned delegation is an industry standard
Furthermore, permissioned delegation via verifiable strong cryptographic signature is a well-known, tested feature in smart contracts of distributed blockchain-based applications.

## Additional Resources
* [Source name](http://www...) with description


## Glossary
* **Delegate**: An AccountId that has been granted specific permissions by its Delegator.
* **Delegator**: An AccountId that has granted specific permissions to a Delegate.
* **Account**: a collection of key pairs which can have a specific token balance.
* **AccountId**: A 32-byte number that is used to refer to an on-chain Account.
* **Provider**: A company or individual operating an on-chain Delegate Account in order to post MRC transactions on behalf of other Accounts.  Provider Accounts will have one or more token balances.
* **End User**: Groups or individuals that own an Account that is not a Provider Account.

