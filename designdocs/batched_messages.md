# Batched Messages

## Table of Contents
* [Context and Scope](#context-and-scope)
* [Problem Statement](#problem-statement)
* [Goals and Non-Goals](#goals-and-non-goals)
* [Proposal](#proposal)
* [Benefits and Risks](#benefits-and-risks)
* [Alternatives and Rationale](#alternatives-and-rationale)
* [Glossary](#glossary)

## Context and Scope
This design document describes batched messages and APIs for sending and retrieving them.

## Problem Statement
In order to reduce costs for announcers of messages on-chain as well as reduce network congestion, announcers collate messages into batches of the same type of message and announce the batch location on-chain, instead of announcing each individual message.

## Goals and Non-Goals
This specifies how batches are to be announced on chain; what is required and how a batch may be partially verified based on on-chain information.

This document does not describe the types of DSNP messages that would be in batch announcements. Batch announcements must reference a schema through its ID number. A schema may or may not describe a DSNP message.

This document does not discuss validation of batch file format located at the URI in the announcement, since
the file format cannot be verified on-chain. For details about batch files themselves, see [DSNP Spec: Batch Publications](https://spec.dsnp.org/DSNP/BatchPublications).

## Proposal
* All names are placeholders and may be changed.
* Types may change as needed during implementation phase
* Errors in the extrinsic(s) must have different, reasonably-named error enums for each type of error for ease of debugging.

### Constants
* `BatchPageSizeMax` - the maximum number of batch announcements that will be returned by the `get_batches` query
* `BatchSizeMinBytes` - the minimum possible size of a valid batch file with 1 row

### Enums
* `BatchFormat` - a list of supported formats for batch files. Currently only Parquet file format is supported, however,
other formats are being considered.

### Types
* `BatchAnnouncementParams<T:Config>`: generic
    * `batch_uri`:`Vec<u8>` the URI of the batch file. Must be an IPFS [CIDv1](https://github.com/multiformats/cid/) URI. Accepted codec, algorithm, and base are to be determined.
    * `message_schema_id`: `SchemaId`  the schema id for the messages in this batch. The `schema_id` must refer to schemas used for batching only.
    * `file_size`: `usize`, the size of the batch file, used to determine message fee as well as to let consumers know what size files to expect.  Must be &gt;= the minimum possible DSNP batch file size.
    * `file_format`: `BatchFormat`, indicator of the file format of the batch file.

The file hash is not included separately. Since the `batch_uri` uses CIDv1 specification, the file hash is already included.

* `BatchAnnouncement`: implements `BatchAnnouncementParams`, returned by `get_batches`

See the [implementation of paging in the messages pallet](https://github.com/LibertyDSNP/mrc/blob/main/common/primitives/src/messages.rs#L26-L58) for comparison.

* `BatchAnnouncementOptions<T:Config>`
    * `msa_id`:  `Option<MsaId>`, the announcer's MSA id.  Pass None() to get all announcements.
    * `from_block`: `<T::BlockNumber>`, retrieve messages starting at the given block number (inclusive)
    * `to_block`: `<T::BlockNumber>`, retrieve messages ending at the given block number (inclusive)
    * `from_index`: `u32`, starting message index
    * `page_size`: `usize`, retrieve `page_size` messages at a time, up to configured `T::BatchPageSizeMax`. If 0, return `T::BatchPageSizeMax` results

* `BatchAnnouncementResult`
    * `has_next`: `uint32`, current page number
    * `next_block`: `<T::BlockNumber>` starting block number of next page of results
    * `next_index`: `u32` starting index of next results in `next_block`
    * `results`: `Vec<BatchAnnouncement>`

### File Schema
The batch file schema could look like the following:

```json
{
  "batch_uri": Vec<u8>,
  "message_schema_id": SchemaId,
  "file_size": usize,
  "file_format": BatchFormat,
  "announcer_msa_id": MsaId,
  "messages": [Message]
}
```

### Extrinsics
#### announce_batch(origin, batch_announcement_params)
Creates and posts a new batch announcement message on chain, using the batch message Schema Id. This Schema Id must already be registered and will need to be fetched by the extrinsic.  The transaction fee is determined in part by the file size.

* **Parameters**
  * origin:  required for all extrinsics, the caller/sender.
  * `batch_announcement_params`: `BatchAnnouncementParams`, the parameters to use in the batch announcement.

* **Event**:  `Event::<T>::BatchAnnounced(schema_id, msa_id, file_size, batch_uri)`
* **Restrictions**:
  * origin's `msa_id` must have capacity to post the transaction during the current epoch

### Custom RPCs

#### get_batches(options)
Retrieves paged batch announcements that have not been garbage-collected which meet `options` criteria.  The `msa_id` does not need to be active.

* **Parameters**
  * `options`: `BatchAnnouncementOptions` containing batch announcement criteria

* **Returns**
  * `None()` if no messages meet the criteria.
  * `Some(Vec<BatchAnnouncement>)`, in descending block-transaction order


### Benefits and Risks
Please see [Batching Source Dependent Messages With Delegation](https://forums.projectliberty.io/t/04-batching-source-dependent-messages-with-delegation/216), for discussion about the benefits of announcing batch files on chain rather than all types of user-created messages.

One risk is that providers on MRC could simply register a new schema and announce batches "unofficially". We have not decided whether to let everyone with enough token to register a schema. Other MRC participants would need to first learn about and evaluate new schemas, then update their software to consume a new message type.

Another risk, mentioned in the Alternatives and Rationale section, is that providers would announce smaller batches than the actual batch file sizes. Earnest MRC participants, such as indexers, will quickly learn this announcer is not reliable and ignore batches marked with that announcer's MsaId.

### Alternatives and Rationale
We discussed whether a batch message itself can be delegated, but this would have complicated things and we cannot come up with a use case for delegating batches. It also violates the idea of users delegating explicitly to every provider that performs a service for them, which is a fundamental value we want to apply to the network.

We discussed whether to allow URLs such as HTTP/HTTPS or other URLs and instead opted for content-addressable URIs (CIDv1) which can be resolved by some other service.  This allows us to put the file hash directly into a URI.  It reduces message storage because we don't have to include both a URL and a file hash. A file hash is necessary as a check against file tampering.

We revisited the idea of whether it really is necessary to include a file size. We will be charging a premium for larger files, however, there will be per-byte discount for larger files in order to create an incentive for posting batches while reducing the incentive for announcers to allow spam. Although the processing and downloading time for enormous files also serves as a disincentive for spam, we feel it would not be sufficient.

Despite the fact that announcers can lie abut the file size, the file_size parameter also serves as an on-chain declaration that not only allows consumers of batches to quickly discover if a batch announcer was honest, but the file requestor can know in advance when to stop requesting data.

### Glossary
* *IPFS* [InterPlanetary File System](https://docs.ipfs.io/), a decentralized file system for building the next generation of the internet
* *CID* [Content IDentifier](https://github.com/multiformats/cid/), Self-describing content-addressed identifiers for distributed systems
* *MsaId* [Message Source Account ID](https://github.com/LibertyDSNP/mrc/blob/main/designdocs/ACCOUNTS.md) an identifier for a MSA.
