# Provider Registration

## Table of Contents
* [Context and Scope](#context-and-scope)
* [Problem Statement](#problem-statement)
* [Goals and Non-Goals](#goals-and-non-goals)
* [Proposal](#proposal)
* [Benefits and Risks](#benefits-and-risks)
* [Alternatives and Rationale](#alternatives-and-rationale)
* [Glossary](#glossary)

## Context and Scope
In order to properly implement Frequency, we need to describe how service providers
will participate on the network.

## Problem Statement
Service providers are an integral part of how Frequency functions. However, there are
some open questions about how they operate, questions that this document hopes
to provide clarity on.

Among those questions are:

1. Who can become a provider?
1. How does a user become a provider?
1. What are the attributes of a provider?
1. How will providers be identified to other network actors?
1. How does one stop being a provider?
1. Can provider status be revoked from a provider?

## Goals and Non-Goals
This document will discuss the ways in which service providers can register with
and participate in the Frequency network.

This document will not discuss the specifics of how providers will process
service requests from users on the network. This document will also not discuss
specifics around creating and honoring transactions between users.

## Proposal
The basic workflow for provider registration is as follows:
1. User stakes balance to become a provider. This is called a "Registration Fee".
1. The "Registration Fee" is credited to the Chain Treasury (TBD).
1. The provider's MSA id is added to storage on chain.

Should a provider choose at any point to stop providing services, it can issue a
request to remove itself from the provider registry:
1. The provider issues an event that it will stop providing services.
1. The provider's MSA id is removed from storage.
1. Any delegations associated with that provider should be revoked.

Please note:
* All names are placeholders and may be changed.
* Types may change as needed during implementation phase
* Errors in the extrinsic(s) must have different, reasonably-named error enums
  for each type of error for ease of debugging.

### Types
* `ProviderRegistrationParams<T: Config>`, the arguments used to emit registration event.
  * `provider_msa_id`: `MsaId`
  * `provider_metadata`: `ProviderMetadata`
* `ProviderDeregistrationParams<T:Config>`, the arguments used to deregister a provider.
  * `provider_msa_id`: `MsaId`
* `ProviderMetadata`
  * `name`: `Vec<u8>`
* `Provider`
  * `provider_msa_id`: `MsaId`
  * `metadata`: `ProviderMetadata`

#### Events
* `ProviderRegistrationEvent<T: Config>`, the resource that exists on-chain
  * `block_number`: `BlockNumber`
  * `provider_msa_id`: `MsaId`
* `ProviderDeregistrationEvent<T:Config>`, the resource that exists on-chain
  * `provider_msa_id`: `MsaId`

#### Storage
* `ProviderRegistry<T: Config>`: `StorageMap<MsaId, Provider>`
  * Stores registered providers and provides lookup functionality via `MsaId`.
    Existence in this storage structure implies that a provider's fee has been
    paid and their registration was otherwise successful.

### Extrinsics
#### register_provider(origin, registration_params)
Creates and posts a `ProviderRegistrationEvent`. The `MsaId`
included in the registration must already exist.

This extrinsic is responsible for storing the registered provider in the
`ProviderRegistry`.

* **Parameters**
  * `origin`: `Origin`  required for all extrinsics, the caller/sender.
  * `registration_params`: `ProviderRegistrationParams`, the parameters to use for registration.
* **Event**:  `Event::<T>::ProviderRegistrationEvent(provider_msa_id)`
* **Restrictions**:
  * `origin`'s `msa_id` must have capacity to post the transaction (including fee) during the current epoch.

#### deregister_provider(origin, deregistration_params)
Creates and posts a `ProviderDeregistrationEvent`. The `MsaId`
included in the event must already exist.

This extrinsic is responsible for deleting the registered provider's `MsaId` from the
`ProviderRegistry`.

* **Parameters**
  * `origin`: `Origin`  required for all extrinsics, the caller/sender.
  * `deregistration_params`: `ProviderDeregistrationParams`, the parameters to use in the deregistration.
* **Event**:  `Event::<T>::ProviderDeregistrationEvent(provider_msa_id)`
* **Restrictions**:
  * `origin`'s `msa_id` must have capacity to post the transaction during the current epoch.

### Custom RPCs
#### get_provider(provider_msa_id)
Retrieves a single provider. The `provider_msa_id` should belong to a registered
provider.

* **Parameters**
  * `provider_msa_id`: `MsaId` the `MsaId` of the provider in question.

* **Returns**
  * `None()` if no messages meet the criteria.
  * `Some(Provider)`

## Benefits and Risks
### Provider Registry and Info
This structure allows for easy provider lookups, in the event actors on the
chain need to query provider information. As of the time of writing this
document, the only provider information required is a name (non-unique). In the
future, we may include other information like domain, logo, business address, etc.

### Provider Verification
There should be a way to verify whether providers are legitimate entities. As of
now, there is no authority that we can query to verify providers, nor is there a
real-world counterpart that can verify provider information.

In the future, we may require a business domain (w/ SSL certificate) to grant
provider status. However, doing so would bar entities that do not have
traditional business attributes from participating on the network as service
providers ("mom and pop" local businesses and/or "freelancers").

So to allow inclusion for all actors, it may be that the best way of verifying
legitimateness is through a fee that is high enough to discourage malicious actors.

### Archival Provider Information
If consumers want to examine provider registration / deregistration events, they
must fetch them from an archival node. This document does not outline an RPC for
fetching registration / deregistration events.

## Alternatives
### Provider Fees
This document assumes that registration fees will be a fixed amount. It may be
prudent, in the future, to determine whether or not registrations are fixed or
vary based on the amount of verifable information given on a registration
attempt. The latter may allow us to place more trust into providers that do not
have traditional business attributes.

## Glossary
TBD.
