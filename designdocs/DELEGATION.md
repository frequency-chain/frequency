# Delegations
Please see the [Design Doc README](https://github.com/LibertyDSNP/meta/blob/main/DESIGN_DOCS.md) for details on what goes in each section.

## Table of Contents (Optional)
* [Glossary](https://github.com/LibertyDSNP/meta#overview)
* [Context and Scope](https://github.com/LibertyDSNP/meta#installation)
* [Problem Statement](https://github.com/LibertyDSNP/meta#dependenciesrequirements)
* [Goals and Non-Goals](https://github.com/LibertyDSNP/meta#configuration)
* [Proposal](https://github.com/LibertyDSNP/meta#examples)
* [Benefits and Risks](https://github.com/LibertyDSNP/meta#roadmap)
* [Alternatives and Rationale](https://github.com/LibertyDSNP/meta#support)
* [Additional Resources](https://github.com/LibertyDSNP/meta#contributing)

## Glossary
* **Delegate**: An AccountId that has been granted specific permissions by its Delegator.
* **Delegator**: An AccountId that has granted specific permissions to a Delegate.
* **Account**: a collection of key pairs which can have a specific token balance.
* **AccountId**: A 32-byte number that is used to refer to an on-chain Account.
* **Provider**: A company or individual operating an on-chain Delegate Account in order to post MRC transactions on behalf of other Accounts.  Provider Accounts will have one or more token balances.
* **End User**: Groups or individuals that own an Account that is not a Provider Account.

## Context and Scope
This document describes how a delegation is created and validated on chain.
Delegations can be used to perform tasks on behalf of another AccountId
Some examples of delegated actions and delegated permissions are given.
It's expected that the actions and permissions that are implemented for delegation will evolve as needed.

## Problem Statement
The primary motivation for delegation is to support End Users of the DSNP platform, however, it is expected that delegation will be used in other ways.

Market research makes it clear that End Users are extremely reluctant to pay to use applications, particularly social networks.
This means there needs to be some way to onboard End Users and relay their activity through the DSNP platform without charging them.
On Ethereum and now on MRC, the use of authorized Delegates enables the creation of End User Accounts as well as processing and storing user messages and other data for the End Users, paid for by a Provider, who can recoup these costs by other means (outside the scope of this Design Document).
The vast majority of this activity will not reside on chain, however, MRC needs to be able to coordinate the exchange of data.

## Goals and Non-Goals
Delegation, roughly speaking, must allow all Create, Read, Update and Delete (CRUD) operations by a Delegating Account to fulfill the purpose of giving Account holders proper authority over their Delegates.  Put another way, delegation must have the following properties:
* **Authorizable** - delegations must be able to be authorized with specific permissions by Accounts.
* **Verifiable** - verifiability allows others to check that Providers are doing things only when authorized and only what they are authorized to do.
* **Transparent** - in order to verify Delegates, the delegations must be readable by anyone, in order to maximize opportunities to police Provider actions.
* **Changeable** - a Delegate's permissions must be able to be changed by the Delegator to give Account holders control over what tasks are permitted to the Delegate.
* **Revocable** - a Delegate's permissions must be able to be revoked by the Delegator to give Account holders the ability to withdraw permissions completely from the Delegate.

## Proposal
The proposed solution is to give End Users the ability to transparently authorize Delegates on chain and control what activities are delegated.

## Benefits and Risks
## Alternatives and Rationale
## Additional Resources

* [Source name](http://www...) with description
