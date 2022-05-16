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
* `MessageType` - all values specified in [DSNP Spec Announcement Types](https://spec.dsnp.org/DSNP/Announcements.html)

### Types
* `BatchAnnouncementParams<T:Config>`: generic
    * `batch_url`:`Vec` the URL of the batch file.  Must be IPFS or HTTPS URL. The URL must be correctly formatted.
    * `message_type`: `MessageType`  the type of messages in this batch
    * `file_size`: `usize`, the size of the batch file, used to determine message fee.  Must be &gt;= the minimum possible DSNP batch file size.
    * `file_hash`: `<T::Hash>`, the hash of the batch file. Must not be 0 hash
    * `msa_id`: `MsaId`, the id to use for the announcer.  Must exist and be active
    * `signature`: `<T::Signature>`, the signature over `file_hash`, using the origin `AccountId`.  Signature must be valid for the file hash + origin

* `BatchAnnouncement`: implements `BatchAnnouncementParams`, returned by `get_batches`

* `BatchAnnouncementOptions<T:Config>`: struct
    * `msa_id`:  `MsaId`, the announcer's MSA id.  If 0, get all announcements
    * `block_number`: `<T::BlockNumber>`, retrieve messages starting at the given block number (inclusive). If 0, get all messages
    * `page_size`: `usize`, retrieve `page_size` messages at a time, up to configured `T::BatchPageSizeMax`. If 0, return `T::BatchPageSizeMax` results.

* `BatchAnnouncementResult`: struct
    * `page`: `uint32`, current page number
    * `results`: `BoundedVec<BatchAnnouncement, page_size>`


### Extrinsics
#### announce_batch(batch_announcement_params)
Creates and posts a new batch announcement message on chain.

* **Parameters**
  * `batch_announcement_params`: `BatchAnnouncementParams`, the parameters to use in the batch announcement.

* **Event**:  `Event::<T>::BatchAnnounced(schema_id file_size, file_hash, msa_id, signature)`
* **Restrictions**:
  1. origin must own `msa_id`.
  2.`msa_id` account must have capacity to post the transaction during the current epoch

### Custom RPCs

#### get_batches(options)
Retrieves paged batch announcements that have not been garbage-collected which meet `options` criteria.  The `msa_id` does not need to be active.

* **Parameters**
  1. `options`: `BatchAnnouncementOptions` containing batch announcement criteria

* **Returns**
  * `None()` if no messages meet the criteria.
  * `Some(BoundedVec<BatchAnnouncement, page_size>)`, in descending block-transaction order

