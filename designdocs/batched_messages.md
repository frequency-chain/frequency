# Batched Messages

## Table of Contents

- [Context and Scope](#context-and-scope)
- [Problem Statement](#problem-statement)
- [Goals and Non-Goals](#goals-and-non-goals)
- [Proposal](#proposal)
- [Benefits and Risks](#benefits-and-risks)
- [Alternatives and Rationale](#alternatives-and-rationale)
- [Glossary](#glossary)

## Context and Scope

This design document describes message schemas. It also will describe
batchability as a logical construct derived from schemas.

We will also be updating the APIs for creating schemas.

## Problem Statement

In order to reduce costs for announcers of messages on-chain as well as reduce
network congestion, announcers collate messages into batches of the same type of
message and announce the batch location on-chain, instead of announcing each
individual message.

However, this idea does not go far enough. Batching allows us to support posting
massive amounts of data, but it would still be expensive to post data of this
size on chain.

We can leverage off chain storage to make posting large message collections cheap.
This document aims to explore what a system that does that could look like.

## Goals and Non-Goals

This specifies how messages are to be announced on chain; what is required and
how a batch may be partially verified based on on-chain information.

This document specifies how messages can be inferred from both schema format type
and payload location.

This document also specifies how schemas will constrain the shape of off chain
messages.

This document does not describe the types of DSNP Announcements that will be
described by schemas. In theory, any message model can be supported.

This document also does not discuss validation of either model or model type. If
this type of validation is necessary, it should be described elsewhere.

## Proposal

- All names are placeholders and may be changed.
- Types may change as needed during implementation phase
- Errors in the extrinsic(s) must have different, reasonably-named error enums for each type of error for ease of debugging.

### Enums

- `ModelType` - supported serialization formats for message payloads files. Currently only [Parquet](https://parquet.apache.org/docs/) and
  [Avro](https://avro.apache.org/docs/current/) are supported.
- `PayloadLocation` - The location of the payload. Can be either `OnChain` or `IPFS`.
  - `OnChain`
  - `IPFS`
- `Payload`
  - `OnChain`
    - `source`: `MsaId`
    - `payload`: `Vec<u8>`
  - `IPFS`
    - `payload_cidv1`: `Vec<u8>`
    - `payload_byte_length`: `u64`

### Traits

- `Model` - TBD. A common interface for accessing message payload information.
  - Derives `Encode`, `Decode`, `MaxEncodedLen`
  - `max_length`: `SchemaMaxBytesBoundedVecLimit`

### Types

- `Schema<T:Config>`: generic

  - `model_type`: `ModelType` See enum section above.
  - `model`: `Model` Defines the shape of the message payload.
  - `payload_location`: `PayloadLocation` See enum section above.

- `Message<T:Config>`: generic
  - `schema_id`: `u16`
  - `source`: `MsaId` Source of the message.
  - `provider`: `MsaId` Public key of a capacity-providing account
  - `payload`: `Payload` The payload.

(See alternatives section for another way to structure payloads)

### Extrinsics

#### create_schema(origin, schema_params)

Creates and posts a new schema on chain. The transaction fee is determined in part by the model size.

- **Parameters**

  - origin: required for all extrinsics, the caller/sender.
  - `schema_params`: `Schema`, the parameters to use in the batch announcement.

- **Restrictions**:
  - TBD

### Custom RPCs

#### get_schema(schema_id)

Retrieves a `Schema`.

- **Parameters**

  - `schema_id`: `u16` a schema identifier

- **Returns**
  - `None()` if no schemas meet the criteria.
  - `Some(Schema)`

#### MessagesPallet::add(origin, on_behalf_of, schema_id, payload, payload_location)

This existing RPC call will need to change slightly. The `payload` param, at the
of this document's writing, is a `Vec<u8>`. This proposal will turn the
`payload` param's type to `Payload`. It will also add a 5th param for
`payload_location`.

- **Parameters**

  - `origin`: `Origin` A signed transaction origin from the provider
  - `on_behalf_of`: `Option<MessageSourceId>` The msa id of delegate.
  - `schema_id`: `u16` A schema identifier
  - `payload`: `Payload` The message payload

- **Returns**
  - [DispatchResultWithPostInfo](https://paritytech.github.io/substrate/master/frame_support/dispatch/type.DispatchResultWithPostInfo.html) The return type of a Dispatchable in frame.

### Batch as a Logical Construct

We can circumvent defining a batch explicitly if we leverage the model type and
payload location included in the schema.

Parquet files are lists by default, so consumers can assume that a message is
a batch if it has a Parquet model type. In this case, the "batch" will be
located off chain, because storing such a file on-chain would incur significant
cost.

Avro files, on the other hand, have the option of being `record` types (see
[Avro docs](https://avro.apache.org/docs/current/spec.html#schemas)). These files
could be stored either on chain or off chain. If they are on chain, it would
make sense for the file to be small (lower cost). However, they could be large
and stored off chain.

See below to see how the combination of format and location indicate possible
payload types:

```txt
| Model Type | Location         | Example Use Case                      |
-------------------------------------------------------------------------
| Avro       | On-chain         | DSNP Graph Change                     |
| Parquet    | On-chain         | Unknown                               |
| Avro       | IPFS (Off-chain) | Larger Avro structures                |
| Parquet    | IPFS (Off-chain) | DSNP Broadcast or Reply Announcements |
```

### Benefits and Risks

Please see [Batching Source Dependent Messages With Delegation](https://forums.projectliberty.io/t/04-batching-source-dependent-messages-with-delegation/216), for discussion about
the benefits of announcing batch files on chain rather than all types of
user-created messages.

One risk is that providers on Frequency could simply register a new schema and
announce batches "unofficially". We have not decided whether or not to let everyone
with enough token balance register a schema. Other Frequency participants would need to
first learn about and evaluate new schemas, then update their software to
consume a new message type.

There are some upsides to deriving batching logically from existing structures.
One is cost savings. Not having a batch structure means we don't need to worry
about any on-chain computation associated with batch messages -- we simply look
at the format and location on the parent schema and we can deduce whether the
file is singular or plural.

### Alternatives and Rationale

We discussed whether a batch message itself can be delegated, but this would
have complicated things and we cannot come up with a use case for delegating
batches. It also violates the idea of users delegating explicitly to every
provider that performs a service for them, which is a fundamental value we want
to apply to the network.

We discussed whether to allow URLs such as HTTP/HTTPS or other URLs and instead opted for content-addressable URIs (CIDv1) which can be resolved by some other service. This allows us to put the file hash directly into a URI. It reduces message storage because we don't have to include both a URL and a file hash. A file hash is necessary as a check against file tampering.

We revisited the idea of whether it really is necessary to include a file size. We will be charging a premium for larger files, however, there will be per-byte discount for larger files in order to create an incentive for posting batches while reducing the incentive for announcers to allow spam. Although the processing and downloading time for enormous files also serves as a disincentive for spam, we feel it would not be sufficient.

Despite the fact that announcers can lie about the file size, the file_size
parameter also serves as an on-chain declaration that not only allows consumers
of batches to quickly discover if a batch announcer was honest, but the file
requestor can know in advance when to stop requesting data.

#### Changes to `get_messages_by_schema_id`

The `MessagesPallet::get_messages_by_schema_id` RPC returns a paginated
`MessageResponse`. It is possible that this document will change the structure
of the `MessageResponse` to be more like the following:

```rust
pub struct MessageResponse<AccountId, BlockNumber> {
	/// Serialized data in the schemas.
	#[cfg_attr(feature = "std", serde(with = "as_hex_option", skip_serializing_if = "Option::is_none", default))]
	pub payload: Option<Vec<u8>>,
  /// The content address for an IPFS payload
	#[cfg_attr(feature = "std", serde(skip_serializing_if = "Option::is_none", default))]
	pub cid: Option<Vec<u8>>,
  ///  Offchain payload length (IPFS).
	#[cfg_attr(feature = "std", serde(skip_serializing_if = "Option::is_none", default))]
	pub payload_length: Option<u32>,
	/// Message source account id of the Provider.
	pub provider_msa_id: MessageSourceId,
	///  Message source account id (the original source).
	#[cfg_attr(feature = "std", serde(skip_serializing_if = "Option::is_none", default))]
	pub msa_id: Option<MessageSourceId>,
	/// Index in block to get total order
	pub index: u16,
	/// Block-number for which the message was stored.
	pub block_number: BlockNumber,
}
```

### Glossary

- _IPFS_ [InterPlanetary File System](https://docs.ipfs.io/), a decentralized file system for building the next generation of the internet
- _CID_ [Content IDentifier](https://github.com/multiformats/cid/), Self-describing content-addressed identifiers for distributed systems
- _MsaId_ [Message Source Account ID](https://github.com/frequency-chain/frequency/blob/main/designdocs/accounts.md) an identifier for a MSA.
