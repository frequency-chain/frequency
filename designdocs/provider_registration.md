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
1. The provider issues an announcement that it will stop providing services.
1. The provider's MSA id is removed from storage.


Please note:
* All names are placeholders and may be changed.
* Types may change as needed during implementation phase
* Errors in the extrinsic(s) must have different, reasonably-named error enums
  for each type of error for ease of debugging.

### Types
* `ProviderRegistrationParams<T: Config>`, the arguments used to register an announcement.
  * `provider_msa_id`: `MsaId`
  * `registration_fee`: `Balance`
  * `provider_metadata`: `ProviderAnnouncementMetadata`
* `ProviderRegistrationAnnouncement<T: Config>`, the resource that exists on-chain
  * `block_number`: `BlockNumber`
  * `provider_msa_id`: `MsaId`
  * `provider_metadata`: `ProviderAnnouunucementMetadata`
* `ProviderUnregistrationParams<T:Config>`, the arguments used to unregister an announcement.
  * `provider_msa_id`: `MsaId`
* `ProviderUnregistrationAnnouncement<T:Config>`, the resource that exists on-chain
  * `provider_msa_id`: `MsaId`
* `ProviderAnnouncementMetadata`
  * `name`: `Vec<u8>`
* `Provider`
  * `provider_msa_id`: `MsaId`
  * `name`: `Vec<u8>`
* `ProviderRegistrationAnnouncementOptions<T:Config>`
  * `provider_msa_id`:  `Option<MsaId>`, the announcer's MSA id.  Pass None() to get all announcements.
  * `from_block`: `<T::BlockNumber>`, retrieve messages starting at the given block number (inclusive)
  * `to_block`: `<T::BlockNumber>`, retrieve messages ending at the given block number (inclusive)
  * `from_index`: `u32`, starting message index
  * `page_size`: `usize`, retrieve `page_size` messages at a time, up to configured `T::PageSizeMax`. If 0, return `T::PageSizeMax` results

#### Storage
* `ProviderRegistry<T: Config>`: `StorageMap<MsaId, Provider>`
  * Stores registered providers and provides lookup functionality via `MsaId`.
    Existence in this storage structure implies that a provider's fee has been
    paid and their registration was otherwise successful.
* `ProviderRegistrationFees<T: Config>`: `StorageMap<MsaId, Balance>`
  * Stores registration fees paid by providers. This structure can be used to
    verify that a provider has indeed paid their registration fee.

### Extrinsics
#### register_provider(origin, registration_params)
Creates and posts a `ProviderRegistrationAnnouncement` on chain. The `MsaId`
included in the announcement must already exist.

This extrinsic is responsible for storing the registered provider in the
`ProviderRegistry` as well as the provider's fee in the `ProviderRegistrationFees`.

* **Parameters**
  * `origin`: `Origin`  required for all extrinsics, the caller/sender.
  * `registration_params`: `ProviderRegistrationParams`, the parameters to use in the registration announcement.
* **Event**:  `Event::<T>::ProviderRegistered(provider_msa_id, provider_metadata)`
* **Restrictions**:
  * `origin`'s `msa_id` must have capacity to post the transaction (including fee) during the current epoch

#### unregister_provider(origin, unregistration_params)
Creates and posts a `ProviderUnregistrationAnnouncement` on chain. The `MsaId`
included in the announcement must already exist.

This extrinsic is responsible for deleting the registered provider's `MsaId` from the
`ProviderRegistry`.

* **Parameters**
  * `origin`: `Origin`  required for all extrinsics, the caller/sender.
  * `unregistration_params`: `ProviderUnregistrationParams`, the parameters to use in the unregistration announcement.
* **Event**:  `Event::<T>::ProviderUnregistered(provider_msa_id, provider_name)`
* **Restrictions**:
  * `origin`'s `msa_id` must have capacity to post the transaction during the current epoch

### Custom RPCs
#### get_provider_announcement(provider_msa_id)
Retrieves a single provider announcement. The `provider_msa_id` can belong to a
registered or unregistered provider.

* **Parameters**
  * `provider_msa_id`: `MsaId` the `MsaId` of the provider in question.

* **Returns**
  * `None()` if no messages meet the criteria.
  * `Some(ProviderRegistrationAnnouncement)`

#### get_provider_announcements(options)
Retrieves paged registration announcements that have not been garbage-collected
which meet `options` criteria. The `provider_msa_id`s can belong to
registered or unregistered providers.

* **Parameters**
  * `options`: `ProviderRegistrationAnnouncementOptions` containing regitration announcement criteria

* **Returns**
  * `None()` if no messages meet the criteria.
  * `Some(Vec<ProviderRegistrationAnnouncement>)`, in descending block-transaction order

## Benefits and Risks
### Provider Registry and Info
This structure allows for easy provider lookups, in the event actors on the
chain need to query provider information. As of the time of writing this
document, the only provider information required is a name (non-unique). In the
future, we may include other information like domain, logo, business address, etc.

### Provider Fees
This document leaves the amount to charge entities for registering as an open
question. It may be prudent, in the future, to determine whether or not
registrations are fixed or vary based on the amount of verifable information
given on a registration attempt. The latter may allow us to place more trust
into providers that do not have traditional business attributes.

### Recording Provider Fees
This document defines an onchain map for storing registraion fees by `MsaId`.
This may be useful for granting insight into how much was paid to register in
the event fees differ by provider. It also may give us visibilty into when and
how much was paid in the event the chain treasury is opaque.

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

## Alternatives and Rationale
#### Privacy
A full copy of provider metadata is recorded to the chain when a provider is
registered. However, that information has been intentionally left out of the
unregister announcement because it seems plausible that a provider that chooses
to step down may not want their information publicized (to allow for a silent
exit).

## Glossary
TBD.
