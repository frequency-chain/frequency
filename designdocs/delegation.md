# Delegations

This document describes the permissioned delegation of actions, largely, but not limited to, account creation and announcing messages by the owner of an MSA id on chain on behalf of the owner of another MSA id.

## Table of Contents

- [Context and Scope](#context-and-scope)
- [Problem Statement](#problem-statement)
- [Goals and Non-Goals](#goals-and-non-goals)
- [Proposal](#proposal)
- [Benefits and Risks](#benefits-and-risks)
- [Alternatives and Rationale](#alternatives-and-rationale)
- [Glossary](#glossary)

## Context and Scope

This document describes how a delegation is created and validated on chain, and outlines an API.
Delegation to one account can be used to perform tasks on behalf of another account.
Some examples of delegated actions and delegated permissions are given.
It's expected that the actions and permissions that are implemented for delegation will evolve as needed.

## Problem Statement

The primary motivation for delegation is to support End Users of the DSNP platform, however, it is expected that delegation will be used in other ways.

Market research makes it clear that End Users are extremely reluctant to pay to use applications, particularly social networks.
This means there needs to be some way to onboard End Users and relay their activity through the DSNP platform without charging them.
The use of authorized Delegates enables the creation of End User accounts as well as processing and storing user messages and other data for the End Users, paid for by a Provider, who can recoup these costs by other means (outside the scope of this Design Document).
The vast majority of this activity will not reside on chain, however, Frequency needs to be able to coordinate the exchange of data, and to securely allow an End User or any other type of account holder to manage their Delegates.
The delegation is managed by assigning each account, called a Message Source Account or MSA, an ID number, called an MsaId.

## Goals and Non-Goals

Delegation, roughly speaking, must allow all Create, Read, Update and Delete (CRUD) operations by a Delegating MSA to fulfill the purpose of giving other MSAs proper authority over their Delegates.
Put another way, delegation must have the following properties:

- **Authorizable** - delegations can be authorized with specific permissions by MSAs.
- **Verifiable** - there is a way to check that Providers are doing things only when authorized and only what they are authorized to do.
- **Transparent** - delegations can be readable by anyone, in order to maximize opportunities to police Provider actions.
- **Changeable** - a Delegator can change Provider permissions to give MSAs control over what tasks are permitted to the Provider. https://github.com/frequency-chain/frequency/blob/main/designdocs/provider_permissions.md
- **Revocable** - a Delegator can withdraw permissions completely from the Provider.

### Non-Goals

- Doesn't cover handling the retirement of an MSA id, which is a possible future feature and would affect delegation validation and queries.
- Delegated removal would allow removing any other provider without substituting itself as the new provider. Such an endpoint presents serious enough issues that it should be discussed and designed separately, if it's to be implemented at all.
- Does not specify what the permissions are nor the permissions data type.
- Does not specify a value for pallet constants, only when there should be one. These values should be determined by such factors as storage costs and performance.
- Does not include a "block/ban" feature for delegation, which is under discussion; the belief is that a Provider also ought to be able to permanently refuse service to a given MSA id, which further supports the idea of a mutually agreed upon relationship.

## Proposal

The proposed solution is to give End Users the ability to create an on-chain MSA id through an authorized provider. End Users can also transparently authorize and manage their own Providers and permissions, either directly using a native token or through an explicitly authorized Provider. Additionally, we allow MSA ids` to be directly purchased using a native token.

### API (extrinsics)

- All names are placeholders and may be changed.
- All extrinsics must emit an appropriate event with all parameters for the call, unless otherwise specified.
- Errors in the extrinsics must have different, reasonably-named error enums for each type of error for ease of debugging.
- "Owner only" means the caller must own the delegatorMSA id.
- Events are not deposited for read-only extrinsic calls.

#### create_sponsored_account_with_delegation

Creates a new MSA on behalf of a delegator and adds the origin held MSA as its provider.

- Parameters:

  1. `add_provider_payload` - this is what the holder of delegator_key must sign and provide to the provider beforehand.
     - `authorized_msa_id` - the provider, of type `MessageSourceId`
  2. `delegator_key` - The authorizing key used to create `proof`
  3. `proof` - The signature of the hash of `add_provider_payload` by the delegator

- Events:
  1. `MsaCreated`
     - `new_msa_id` - id of the newly created MSA
     - `key` - the `delegator_key`
  2. `DelegationGranted`
     - `delegator` - id of the newly created MSA
     - `provider` - id of the MSA help by the provider

#### revoke_delegation_by_provider

Provider revokes its relationship from the specified `delegator` in the parameters. This function allows a provider to control access to its services, for example, in the case of an End User that violates Terms of Service.

- Parameters:

  1. `delegator` - the MSA id of the delegator

- Events:
  1. `ProviderRevokedDelegation`
     - `provider` - id of the MSA held by the delegator
     - `delegator` - id of the MSA held by the provider

#### create

Directly creates an MSA for the origin (caller) without a provider.
This is a signed call directly from the caller, so the owner of the new MSA pays the fees for its creation.

- Events:
  1. `DelegatorRevokedDelegation`
     - `msa_id` - id of the newly created MSA

#### revoke_delegation_by_delegator

A delegator removes its relationship from a provider.
This is a signed call directly from the delegator's MSA.
This call incurs no fees.

- Parameters:

  1. `provider_msa_id` - id of the MSA held by the provider

- Restrictions: **Owner only**.

- Event: `DelegateRemoved`
  1. `delegator` - id of the MSA held by the delegator
  2. `provider` - id of the MSA held by the provider

### Custom RPC endpoints

#### get_msa_keys(msa_id)

Retrieve a list of public keys of up to `MaxPublicKeysPerMsa` size for the provided MSA id, or an empty list if the MSA id does not exist.

- Parameters:
  1. `msa_id`: the MSA id of which associated keys are to be retrieved

#### check_delegations

Validate that a provider can delegate for a list of MSA ids.
This call is intended for validating messages in a batch, so this function would be an all-or-nothing check.
If the permission stored for a given MSA id exceeds the parameter, the check for that MSA id passes.
For example, if a provider has _all_ permissions set, then querying for a subset of permissions will pass.
Verify that the provided provider `provider_msa_id` is a provider of the delegator, and has the given permission value.
Returns `Ok(true)` if provider is valid, `Ok(false)` if not.
Throws an Error enum indicating if either provider or delegator does not exist.

- Parameters:
  1. `delegator_msa_ids`: a list of Delegator ids possible delegators
  2. `provider_msa_id`: the ProviderId to verify

### Storage

- Delegations are stored as a Double-key map of Delegator MSA id --> Provider MSA id. The data stored contains the `Permission` for that relationship:

```rust
    pub(super) type DelegatorAndProviderToDelegation<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        Delegator,
        Twox64Concat,
        Provider,
        Delegation<SchemaId, BlockNumberFor::<T>, T::MaxSchemaGrantsPerDelegation>,
        OptionQuery,
    >;
```

## Benefits and Risks

As stated earlier, one of the primary intended benefits of delegation is to allow feeless account creation and messaging.

There is a risk of abuse with delegation of messages, since this makes it possible for a provider to, for example, modify the End User's messages before batching them. The message sender would have to be caught and the End User must react after the fact, instead of the message sender being technologically prevented from this type of dishonesty.

There is another risk of abuse for any other type of delegated call if the wallet that provides the signing capability does not make it very clear to the End User what they're signing.

## Alternatives and Rationale

### End User pays for existential deposit

We briefly discussed the possibility of requiring a small token deposit to create their account. We decided against this option because:

1. As mentioned above, people don't expect and won't pay to use social media.
2. Onboarding would be a problem; even if they did want to pay even a small amount, getting people access to a token is tremendously difficult at this time, requiring unacceptable tradeoffs.
3. We would be unable to serve people who are unbanked or don't have access to crypto trading platforms.

### dApp Developer pays for existential deposit

One alternative to allow for account creation at no cost to the End User was the dApp developer MSA sends an existential deposit to the account to create it.
We decided against this option for a number of reasons.

1. It could create a potential for abuse and token loss by those creating numerous fake accounts and then removing the dApp Public Key as a provider.
2. We have the ability not to require an existential deposit, and felt this to be a better option in this particular case.

### End user pays to send messages, with no possibility of delegating

An alternative for delegating messaging capabilities was to have each End User pay for their own messages.
This was ruled out as the sole solution because:

1. The average person can't or won't pay to use social media.
2. Making End Users pay to send messages would require people to sign transactions every time they make any updates — all posts, all reactions, all replies, all profile changes, all follows/unfollows, etc. Having to do this would be too annoying for the End User.

This design still includes some direct pay endpoints, so even if an End User did not want to trust a provider, they could still pay for all of their messages if they want to assume the cost of running a node and pay directly.

### Permissioned delegation is an industry standard

Furthermore, permissioned delegation via verifiable strong cryptographic signature is a well-known and tested feature in smart contracts of distributed blockchain-based applications.

### Deferred features

#### An "effective block range" for providers

Including an effective block range in the provider storage data would allow providers to be expired, not just removed. A block range could better support features like Tombstone, blocking, and retiring an MSA id. Effective block range is deferred because those features have not been fully defined.

#### add_provider(delegator, provider, permissions)

Directly adding a provider, with or without a provider's permission, is not to be implemented at this time. The original use case was for a potential wallet app to support browsing and adding providers. Adding/replacing a provider for an existing account with an MSA id could still be done using the delegated methods, `add_self_as_delegate` or `replace_delegate_with_self`. A direct add brought up concerns about potential risks of adding a provider without the provider's knowledge. For example, if the provider has removed the delegator for legitimate reasons, such as if the End User violated the provider's Terms of Service, then the provider ought to be able to prevent them from adding the provider again just by paying for it.

## Glossary

- **Provider**: An MSA that has been granted specific permissions by its Delegator. A company or individual operating an on-chain Provider MSA in order to post Frequency transactions on behalf of other MSAs.
- **Delegator**: An MSA that has granted specific permissions to a Provider.
- **MSA**: Message Source Account. A collection of key pairs which can have a specific token balance.
- **Public Key**: A 32-byte (u256) number that is used to refer to an on-chain MSA and verify signatures. It is one of the keys of an MSA key pair
- **MsaId**: An 8-byte (u64) number used as a lookup and storage key for delegations, among other things
- **End User**: Groups or individuals that own an MSA that is not a Provider MSA.
