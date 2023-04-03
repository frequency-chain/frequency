# Design Documents
To create a new design document, Please see the [Design Doc README](https://github.com/LibertyDSNP/meta/blob/main/DESIGN_DOCS.md) for details on what goes in each section, and use the provided template there.

## Accepted Design Documents

* [Accounts](./accounts.md)
  * [PR](https://github.com/LibertyDSNP/frequency/pull/13)
* [On Chain Message Storage](message_storage.md)
  * [Merged Pull Request](https://github.com/LibertyDSNP/frequency/pull/15)
* [Delegation](./delegation.md)
  * [PR](https://github.com/LibertyDSNP/frequency/pull/14)
* [Message Schema(s)](./schema.md)
  * [Merged Pull Request](https://github.com/LibertyDSNP/frequency/pull/17)
* [Provider Permissions and Grants](./provider_permissions.md)
  * [Merged Pull Request](https://github.com/LibertyDSNP/frequency/pull/150)
* [Provider Registration](./provider_registration.md)
  * [Merged Pull Request](https://github.com/LibertyDSNP/frequency/pull/208)
* [Capacity](./capacity.md)
  * [Merged Pull Request](https://github.com/LibertyDSNP/frequency/pull/426)
* [Stateful Storage](./stateful_storage.md)
    * [PR](https://github.com/LibertyDSNP/frequency/pull/900)
* [Graph Sdk](./graph_sdk.md)
    * [PR](https://github.com/LibertyDSNP/frequency/pull/1159)

## Basic Data Model

There are three core data models in Frequency and each corresponds to a pallet.

- [Message Source Account (MSA)](../pallets/msa/)
  - Represents a pseudonymous identity that can be the source of messages or provide access to the chain to others
  - Answers the question of Who when messages are sent
- [Schemas](../pallets/schemas/)
  - Represents how to form a message and details about where it should be stored.
  - Answers the question of How when messages are sent
- [Messages](../pallets/messages/)
  - The metadata and payload or payload reference that a user sends that matches a particular schema.
  - What and when a message is sent

![Basic Data Model drawio](https://github.com/LibertyDSNP/DesignDocs/blob/main/img/BasicDataModel.drawio.png?raw=true)

## Frequency Glossary

* `AccountId`: A public key that could be a `Token Account` and/or associated with an `MSA`
* `Announcer AccountId`: The `AccountId` that signs a capacity transaction and is associated with an MSA from which capacity will be deducted for that capacity transaction.
* `Announcer MSA`: The `MSA` associated with the `AccountId` that signs a capacity transaction.
* `Delegate` (verb): The action of an `MSA` (the `Delegator`) delegating to a `Provider`. *A verb only. Do not use as a noun!*
* `Delegator`: An `MSA` that has delegated to a `Provider`.
* `MRC`: The old name for Frequency
* `MSA Id`: The 64 bit unsigned integer associated with an `MSA`.
* `MSA`: Message Source Account. A registered identifier with the MSA pallet. `AccountIds` (aka public keys) may only be associated with one `MSA` and that association is immutable.
* `Message`: A message that matches a registered `Schema` (on-chain or off-chain).
* `Payload`: The user data in a `Message` that matches a `Schema`.
* `Provider`: An `MSA` that is registered for being able to be delegated to and being the target of capacity rewards when a person stakes to the network for token rewards.
* `Schema`: A registered data structure and the settings around it.
* `Token Account`: An `AccountId` that is holding tokens.

### External Terms

* `IPFS`: [InterPlanetary File System](https://docs.ipfs.io/), a decentralized content-addressed file system.
* `CID`: [Content IDentifier](https://github.com/multiformats/cid/), Self-describing content-addressed identifiers for distributed systems.

### Banned Terms
* `Delegate` (Noun): Confusing due to being spelled the same as the verb and close to `Delegator`. Replaced with `Provider`.

