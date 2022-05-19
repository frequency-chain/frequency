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
This specifies how batches are to be announced on chain: what is required and how a batch may be partially verified based on on-chain information.

This document does not describe the messages that are supported for batch announcements. Please see the [DSNP Specification](https://spec.dsnp.org/).

This document does not discuss validation of batch file format located at the URL in the announcement, since
the file format cannot be verified on-chain. For details about batch file format, see [DSNP Spec: Batch Publications](https://spec.dsnp.org/DSNP/BatchPublications).

## Proposal
* All names are placeholders and may be changed.
* Errors in the extrinsic(s) must have different, reasonably-named error enums for each type of error for ease of debugging.

### Constants
* `BatchPageSizeMax` - the maximum number of batch announcements that will be returned by the `get_batches` query
* `BatchSizeMinBytes` - the minimum possible size of a valid batch file with 1 row

### Enums
* `BatchFormat` - a list of supported formats for batch files. Currently only Parquet file format is supported, however,
other formats are being considered.

### Types
* `BatchAnnouncementParams<T:Config>`: generic
    * `batch_url`:`Vec<u8>` the URL of the batch file. Must be an IPFS [CIDv1](https://github.com/multiformats/cid/) URL. Accepted codec, algorithm, and base are to be determined.
    * `message_schema_id`: `SchemaId`  the schema id for the messages in this batch. The `schema_id` must refer to schemas used for batching only.
    * `file_size`: `usize`, the size of the batch file, used to determine message fee as well as to let consumers know what size files to expect.  Must be &gt;= the minimum possible DSNP batch file size.
    * `file_format`: `BatchFormat`, indicator of the file format of the batch file.
    * `msa_id`: `MsaId`, the id to use for the batch.  Must exist and be active. Must be owned by origin (sender).

The file hash is not included separately. Since the `batch_url` uses CIDv1 specification, the file hash is already included.

* `BatchAnnouncement`: implements `BatchAnnouncementParams`, returned by `get_batches`

See the [implementation of paging in the messages pallet](https://github.com/LibertyDSNP/mrc/blob/main/common/primitives/src/messages.rs#L26-L58)

* `BatchAnnouncementOptions<T:Config>`
    * `msa_id`:  `MsaId`, the announcer's MSA id.  If 0, get all announcements
    * `from_block`: `<T::BlockNumber>`, retrieve messages starting at the given block number (inclusive)
    * `to_block`: `<T::BlockNumber>`, retrieve messages ending at the given block number (inclusive)
    * `from_index`: `u32`, starting message index
    * `page_size`: `usize`, retrieve `page_size` messages at a time, up to configured `T::BatchPageSizeMax`. If 0, return `T::BatchPageSizeMax` results


* `BatchAnnouncementResult`
    * `has_next`: `uint32`, current page number
    * `next_block`: `<T::BlockNumber>` starting block number of next page of results
    * `next_index`: `u32` starting index of next results in `next_block`
    * `results`: `Vec<BatchAnnouncement>`


### Extrinsics
#### announce_batch(origin, batch_announcement_params)
Creates and posts a new batch announcement message on chain.

* **Parameters**
  * origin:  required for all extrinsics, the caller/sender.
  * `batch_announcement_params`: `BatchAnnouncementParams`, the parameters to use in the batch announcement.

* **Event**:  `Event::<T>::BatchAnnounced(schema_id file_size, file_hash, msa_id, signature)`
* **Restrictions**:
  * origin must own `msa_id`
  * `msa_id` account must have capacity to post the transaction during the current epoch

### Custom RPCs

#### get_batches(options)
Retrieves paged batch announcements that have not been garbage-collected which meet `options` criteria.  The `msa_id` does not need to be active.

* **Parameters**
  * `options`: `BatchAnnouncementOptions` containing batch announcement criteria

* **Returns**
  * `None()` if no messages meet the criteria.
  * `Some(BoundedVec<BatchAnnouncement, page_size>)`, in descending block-transaction order

### Glossary
* *IPFS* [InterPlanetary File System](https://docs.ipfs.io/), a decentralized file system for building the next generation of the internet
* *CID* [Content IDentifier](https://github.com/multiformats/cid/), Self-describing content-addressed identifiers for distributed systems
* *MsaId* [Message Source Account ID](https://github.com/LibertyDSNP/mrc/blob/main/designdocs/ACCOUNTS.md) an identifier for a MSA.
*
