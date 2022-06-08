# Delegator <-> Provider Permissions/Grants

## Context and Scope

MRC enables users(read delegators) to have control over their own data. While providers can send and receive messages on behalf of users, there is need for separation of control via **delegator->provider**, **provider<->delegator** relationships in form of Permissions and Grants respectively. This document describes the design of the two types of relationships and recommendation on how to implement them.

## Problem Statement

Data Access Pattern on MRC, at-minimum, should provide ***PUBLISHER*** and ***RESTRICTED*** ```permissions``` at **delegator->provider**, as well as ***PUBLISH*** and, ***Block***, ```grants``` for specific ```schema_id``` at **provider<->delegator**.This entails users can enable specific permissions for provider to write data on their behalf, while also restricting grants to providers at schema level, rendering providers as restricted. Providers should also be able to opt into publish, on behalf of, users, or block from publication, on behalf of, at schema level. Primarily, the use case can be summarized in following way:

- **As a provider**, I would want to publish data for specific ```schema_id``` on-behalf of a delegator. Defaults to ```publish``` permissions on all schemas registered by provider on behalf of delegator.
- **As a delegator**, I would like to restrict a provider, by allowing a provider to only publish data for specific ```schema_ids``` on-behalf of me.

Note: A publish state would mean that a provider is able to publish data on behalf of a delegator on all public schemas by passing validation. While a restricted state would mean that a provider is not able to publish data on behalf of a delegator on a specific schema, would require additional validation. The default state would be restricted as provider must opt in (permissioned by user) to publish data for specific schema(s) on user's behalf before sending messages for said schema(s).

## Goals and Non-Goals

MRC is a default read only for items stored on and off chain, requiring an explicit process to control writing or publishing of messages via some permissions and grants. Some of the major goals surrounding provider permissions and grants are:

### Goals

**Opt In and Duality**: Providers should register users with MRC and delegate on behalf of them, while also specifically allowing a collection of schema(s) for which delegator provide them full publication rights. This ensures default state of providers is ***Restrict***. Delegators can also choose to restrict providers on per-schema basis by blocking them from publishing data on their behalf. This ensures default state of delegators is ***Block*** for all non provider preferred ```schema_ids```. Duality should be implemented at schema level grants.

**ToS Baked In**: As a part of this design doc, it is recommended to discuss about baking in ***ToS*** for providers and delegators as a part of permission grants by including a hash of ToS unless there is a re-delegation. Such that MRC can also act as proof of specific agreement established between a provider and a delegator.

**Time Bound Grants**: Any grants given or revoked by a delegator (allowing provider to publish or block them for certain duration) or any grants are modified by a provider or delegator are valid for the duration of ***ToS***. This can be a control mechanism in MRC which can be a fixed number for version 1 of this implementation and be extensible via a governance mechanism. This also brings the question about, if not time bounded, does permissions and grants are set till they are explicitly revoked. While un-delegation, definitely is an option for user to remove a provider completely from ever publishing on their behalf.

### Non-Goals

- Does not cover the case where a delegator or provider can restrict reading of data on their behalf.
- MRC enables a valid provider or delegator to be able to read as a default.
- Only covers basic version 1 of permission and grant implementation details.
- Does not cover details of economics, governance mechanism.
- Does not cover details on dynamic expiry time for permissions/grants.

## Proposal

The proposed solution is to provide delegate level permissions and schema level grants to delegators and providers alike. This will enable a provider to publish data on behalf of a delegator, while also allowing a delegator to restrict a provider from publishing data on their behalf for specific schema.

Note: The terminology and implementation are subject to change at issue resolution.

### Permissions

Permission is a generic option for any user. For version 1 of this implementation, the following options are available:

- ***PUBLISHER***: Where a user grants full publication rights to a provider for any schema available to provider via MRC. This can be modified to be called a dsnp publisher where all dsnp related schemas are granted to provider. In other sense this could be super admin permission that can be granted via a governance mechanism.

- ***RESTRICTED***: Where a user grants a provider to publish data on their behalf for specific schema(s) only. This is the default state of a provider on MRC, where a provider has to explicitly provide a list of schema(s) for which they are allowed to publish data on behalf of the user.

An example of permission data structure is as follows:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionType {
    Publisher,
    Restricted,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Permission {
    pub permission_type: PermissionType,
    pub tos_hash: Vec<u8>,
    pub expiry_time: u64,
}
```

### Grants

Grants enable delegators as well as providers to restrict one another from publishing data on specific schema(s). For version 1 of this implementation, the following options are available:

- ***PUBLISH***: Where a delegator grants a provider to publish data on their behalf for specific schema(s) only. This is the default state of a provider on MRC, where a provider has to explicitly provide a list of schema(s) for which they are allowed to publish data on behalf of the delegator. This also enables a delegator to opt in to publish their data.

- ***Block***: When a delegator or provider want to restrict publication of data on specific schema(s). This is default state of any schemas, not authorized by delegator or provider as part of schema grants request.

An example of grant data structure is as follows:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GranType {
    Publish,
    Block,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Grant {
    pub grant_type: GranType,
    pub tos_hash: Vec<u8>,
    pub expiry_time: u64,
}
```

### API (Extrinsic)

- ***delegator_msa***: The MSA of a user.
- ***provider_msa***: The MSA of the provider/app.
- ***Permission***: The generic option for any user. Default mode of operation is ***Restricted*** for any provider unless explicitly added as ***Publisher***.
- ***Grant***: The user level action/result. "A user grants a permission to a provider".
- ***ToS***: The hash of terms of service between a delegator and provider.
- ***expiry***: The expiry time of a permission/grant.
- ***schema_id***: The unique identifier of a registered schema on MRC.

### add_schema_grants()

An extrinsic to allow a provider to request publish write to list of schemas. Rendering them **Restricted** status.

- Parameters:
    1. **provider_msa**: The MSA of the provider/app.
    2. **delegator_msa**: The MSA of a user.
    3. **schemas**: The list of schemas for which the provider wants to grant publish write typically ```Vec<SchemaId>```.
    4. **tos_hash**: The hash of terms of service between a delegator and provider.

- Events: ```SchemaPermissionAdded``` where the event data is ```(delegator_msa, provider_msa, schemas)```.
  
- Restrictions: origin must own provider ```msa_id``` delegated by delegator ```msa_id```.

- Outcomes: Provider permissions are set to **Restricted** and grants have been set for selected schemas.

- Notes: The weights of this extrinsic should account for required weights when revoking grants at schema level via provider/delegator.

### add_mrc_publisher()

An extrinsic to allow (via governance) to set a provider as MRC publisher. This in turn will give all publish rights on all schemas for any delegator delegating to this provider. Rending them **Publisher** status.

- Parameters:
    1. **provider_msa**: The MSA of the provider/app.
    2. **delegator_msa**: The MSA of a user.
    3. **tos_hash**: The hash of terms of service between a delegator and provider.

- Events: ```PublisherPermissionAdded``` where the event data is ```(delegator_msa, provider_msa, tos_hash)```.
  
- Restrictions:
  - This extrinsic is should only be available via governance or via some strict mechanism.
  - Origin must own provider ```msa_id``` delegated by delegator ```msa_id```.

- Outcomes: Provider permissions are set to **Publisher**. This can indicate to by pass schema level grants for delegator at this permission level.

- Notes: The weights of this extrinsic should account for required weights when revoking grants at schema level via provider/delegator. This can then make the following an rpc instead of extrinsic.

### revoke_schema_grants()

An extrinsic (or rpc if revoking is paid off while adding) to allow a provider or delegator to block publishing rights on specific schemas. If a delegator has **Publisher** status, then restricting a schema will render it **Restricted** status on provider.

- Parameters:
    1. **provider_msa**: The MSA of the provider/app.
    2. **delegator_msa**: The MSA of a user.
    3. **schema_ids**: The list of schemas for which the provider/delegator wants to block publishing rights, typically ```Vec<SchemaId>```.

- Restrictions: Origin must own provider ```msa_id``` delegated by delegator ```msa_id```.

- Outcomes: Provider permissions are set to **Restricted**. While schema level grants is set to blocked for a given ```schema_id```.

- Notes:
  - Revocation of a grant should be "pre-paid" (Not staking it, just paying more for the create so the revoke is "covered")
  - If this action is already paid off while [adding schema grants](#add_schema_permissions), this can be done via rpc.
  - Un-delegation of a provider by a delegator should revoke all grants from all schemas for that provider.

## Time bounded permissions

- Expiry time on permissions for version 1 can be a fixed number of blocks set in MRC.
- This expiry time can be update via governance or runtime upgrades. Since this is not important for the scope of the design doc, implementation details can define, how MRC is handling expiry time in first version.
- However, expiry time can be baked in the above extrinsic call and can be set as a parameter.

## Validation

- If a provider has **Publisher** status, then it can publish data on behalf of a delegator for any schema supported by MRC.
- If a provider has **Restricted** status, a check will be required to ensure delegator has given **Publish** grant for a given schema.
- This can further be extended to act as an additional validation on publishing batched messages for a given list of delegators for a specific schema.

## Benefits and Risks

Enabling permissions and grants benefits both user and provider. While providers can be trusted to publish data on behalf of a delegator, it is not always the case and vice versa. Duality of opt in at the grant level solidify the trust relationship.

Some risks are primarily at implementation level, such a storage pattern of such grants, and validation surrounding whether a provider is allowed to publish or not, adds additional overhead. Though this risk is easily minimzed by using a storage pattern that is optimized for the use case.

## Additional Resources

## Discussion Notes from 2022-05-25

- Allowing a site to read my private data
  - Not posting on my behalf, reading my (private) data instead of writing data
  - Key management for reading private data
- Delegation for reading vs writing
- Do we need to have a way for the user to express how their data is used on "3rd party" sites (aka sites you don't have an explicit delegation with)
  - Maybe not MRC, but perhaps at the DSNP layer?
  - DSNP data use policy for each announcement (We already assume friends/followers can see it even on 3rd party sites)
- Do we need extendable/modular permissions or "roles"?
  - Grouping Schemas together somehow?
  - Roles are useful for users, but hide complexity (see Android/iOS permission mess)
  - What if there are 100's of schemas?
  - Could the wallet solve that problem?
- Delegation is a wallet level action
  - Users have a wallet interaction for delegation
- Do schemas have custom permissions?
  - Sub-permissions CRUD? We really only have Create as a permission
- Could service permissions be set at the provider MSA level and the delegation only have exclusions?
  - This is more auto-optin which I think is bad.
- Public "Read" permission (DSNP or MRC?)
  - Robots.txt
  - Could this be on a per service basis?
    - aka I grant the public ability to "read" schema 7 from service A but not schema 7 from service B

- Core delegation model ideas
  - Limited Schema Ids (User allows schema: x, y, z)
    - Want to send a message -> Check delegate -> check schema "permission"
    - Delegation permissions are "stored with" the delegation on-chain
      - This data must be historically immutable and retained
    - Active opt-in
