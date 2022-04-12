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
Delegations can be used to perform tasks on behalf of another StaticId.
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

### Non-Goals
* This doesn't cover retiring a StaticId, which is a possible future feature.
* Although this does specify a direct removal of one's own StaticId, it doesn't specify a delegated version of that.
This would mean one delegate would be able to remove another delegate, not just replacing the delegate with itself.
This presents serious enough issues that it should be discussed and designed separately if it's to be implemented at all.
* We're not specifying here what the permissions are nor the permissions data type.

## Proposal
The proposed solution is to give End Users the ability to create an on-chain StaticId through an authorized delegate, and to transparently authorize and manage their own Delegates and permissions, either directly using a native token or through another delegate that they authorize explicitly. Additionally, we allow StaticIds to be directly purchased using a native token.

### API (extrinsics)
* all names are placeholders and may be changed.
* all extrinsics must emit an appropriate event with all parameters for the call, unless otherwise specified
* errors in the extrinsics must have different, reasonably-named error enums for each type of error.
* Read-only extrinsics can be called by anyone; otherwise, restrictions are as noted.
* Events are not emitted for read-only extrinsic calls.

1. **create_account_with_delegate**(payload) - creates a new StaticId on behalf of an Account, by the caller, and adds the caller as the Account's delegate.  This call is from the delegate account.  The delegate, *not the owner of the new StaticId*, pays the fees.
    * Parameters:
      1. `payload`: authorization data signed by the delegating account.
         1. `data` - this is what the Account owner must sign and provide to the delegate beforehand.
             * `static_id` - the delegate's StaticId, i.e. the caller's StaticId
             * `permissions` a value indicating the permission(s) to be given to the delegate
         2. `signing_key` - The authorizing AccountId, the key used to create `signature`
         3. `signature` - The signature of the hash of `data`
    * Event Emitted:  `IdentityCreated`, with the delegator AccountId, the new StaticId, and the delegate StaticId
    * Restrictions:  The origin account MUST control the static ID that is provided in the payload.
2. **add_self_as_delegate(payload)** - adds the StaticId in the payload as a delegate, to an Account owning `delegator_static_id`
    * Parameters:
      1. `payload`: authorization data signed by the delegating account
          1. `data` - this is what the Account owner must sign and provide to the delegate beforehand.
              * `delegate_static_id` - the delegate's StaticId, i.e. the caller's StaticId
              * `permissions` a value indicating the permission(s) to be given to the delegate
          2. `signing_key` - The authorizing AccountId, the key used to create `signature`
          3. `signature` - The signature of the hash of `data`
    * Event emitted: `DelegateAdded` with the `signing_key`, `static_id`, and `delegate_static_id`
    * Restrictions:  Caller/origin MUST control `delegate_static_id`, and the `signing_key` Account MUST control `static_id`.
3. **replace_delegate_with_self(payload)** - by signed authorization of owner of `delegator`, `delegate` is removed as a delegate of `delegator` and replaced with `new_delegate_static_id`
    * Parameters:
        1. `payload`: authorization data signed by the delegating account
            1. `data` - this is what the Account owner must sign and provide to the delegate beforehand.
                * `new_delegate` - the new delegate's StaticId, i.e. the caller's StaticId
                * `old_delegate` - the StaticId of the delegate to be replaced.
                * `static_id` - the StaticId of the authorizing Account.
                * `permissions` a value indicating the permission(s) to be given to the *new* delegate
            2. `signing_key` - The authorizing AccountId, the key used to create `signature`
            3. `signature` - The signature of the hash of `data`
    * Event emitted: `DelegateReplaced` with the `signing_key`, `static_id`, and `old_delegate` and `new_delegate`
    * Restrictions:  Caller/origin MUST control `new_delegate`, and the `signing_key` Account MUST control `static_id`. Also, `old_delegate` MUST also be a delegate of `static_id`.  Caller may use `add_self_as_delegate` if the last condition fails, but this will cost transaction fees, so it behooves them to call `validate_delegate` with no permissions, if unsure.
5. **validate_delegate(delegate, delegator, permissions)** - verify that the provided delegate StaticId is a delegate of the delegator, and has the given permissions. DispatchResult contains true if the validation passes. Errors if delegate or delegator do not exist.
    * Parameters:
     1. `delegate`: the StaticId of the delegate to verify
     2. `delegator`: the StaticId of the delegator
     3. `permissions`: the permissions to check against what is stored for this delegate.  If `permissions` is the `Zero()` value, DispatchResult contains true if they are a delegate, false if not.
6. **get_static_id(static_id)** - retrieve the AccountId for the provided StaticId, or error if it does not exist.
7. **create_static_id()** - directly creates a StaticId for the origin (caller) Account, with no delegates. This is a signed call directly from the caller, so the owner of the new StaticId pays the fees for StaticId creation.
    * Event Emitted: `IdentityCreated`, with the caller's AccountId, the new StaticId, and an empty delegate StaticId.
8. **add_delegate(delegator, delegate, permissions)** - adds a new delegate for an *existing* StaticId.  This is a signed call directly from the delegator's Account.  The *delegator* account pays the fees.
    * Parameters:
        1. `delegator` - the StaticId to add the delegate to
        2. `delegate` - the StaticId of the new delegate
        3. `permissions` - a value indicating the permissions for the new delegate
    * Restrictions:  **Owner only**.
9. **update_delegate(delegator, delegate, permissions)** - changes the permissions for an existing delegator-delegate relationship. This is a signed call directly from the delegator's Account.  The *delegator* account pays the fees.
    * Parameters: the same as `add_delegate`.
    * Restrictions:  **Owner only**.
10. **remove_delegate(delegator,delegate)** - deletes a delegate's entry from the list of delegates for the provided StaticId. This is a signed call directly from the delegator's Account. The *delegator* account pays the fees, if any.
     * **Parameters**:
         1. `delegator` - the StaticId removing the delegate
         2. `delegate` - the StaticId of the delegate to be removed
     * Restrictions:  **Owner only**.
11. **remove_static_id(static_id)** deletes the StaticId from the registry entirely.
    * Restrictions:  Owner or sudoer


## Benefits and Risks
As stated earlier, one of the primary intended benefits of delegation is to allow feeless account creation and announcing.

There is a risk of abuse with delegation of announcements, since this makes it possible for a delegate to, for example, modify the End User's messages before batching them. The announcer would have to be caught and the End User must react after the fact, instead of the announcer being technologically prevented from this type of dishonesty.

There is another risk of abuse if there is a wallet that does not make very clear what the End User is signing.

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
* **Delegate**: An StaticId that has been granted specific permissions by its Delegator.
* **Delegator**: An StaticId that has granted specific permissions to a Delegate.
* **Account**: a collection of key pairs which can have a specific token balance.
* **AccountId**: A 32-byte number that is used to refer to an on-chain Account.
* **StaticId**: A 32-byte number used as a lookup and storage key for delegations, among other things
* **Provider**: A company or individual operating an on-chain Delegate Account in order to post MRC transactions on behalf of other Accounts.  Provider Accounts will have one or more token balances.
* **End User**: Groups or individuals that own an Account that is not a Provider Account.

