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
This design document describes batched messages and the API for sending them.

## Problem Statement
In order to reduce costs for announcers of messages on-chain as well as reduce network congestion, we have announcers collate messages into batches of the same type of message and announce the batch location on-chain instead of announcing each individual message.

## Goals and Non-Goals
This specifies how batches are to be announced on chain: what is required and how it may be partially verified based on on-chain information.

This document does not describe what types of messages are supported for batches.
This document does not discuss batch message format, since file format cannot be verified on-chain. For details about batch file format, see [DSNP Spec: Batch Publications](https://spec.dsnp.org/DSNP/BatchPublications).

## Proposal
* All names are placeholders and may be changed.
* Errors in the extrinsic(s) must have different, reasonably-named error enums for each type of error for ease of debugging.

### Types
* `BatchAnnouncementParams<T>`: generic
    * `batch_url`:`str` The URL of the batch file.
    * `schema_id`: `SchemaId`, the schema id of the message type contained in the batch.
    * `file_size`: `usize`, the size of the batch file, used to determine message fee.
    * `file_hash`: `Hash<T>`, the keccak-256 hash of the batch file.
    * `msa_id`: `MsaId`, the id to use for the announcer.
    * `signature`: `Signature<T>`, the signature over `file_hash`, using the `AccountId` associated with `msa_id`.

* `BatchAnnouncement`: struct, implements `BatchAnnouncementParams`

* `BatchAnnouncementOption`: struct
    * `schema_id`: `SchemaId`, the schema id of the desired message type. If 0, get all schema_ids
    * `msa_id`:  `MsaId`, the announcer's MSA id.  If 0, get all announcements
    * `block_number`: `BlockNumber`, retrieve messages starting at the given block number (inclusive). If 0, get all messages
    * `page_size`: `usize`, retrieve `page_size` messages at a time

* `BatchAnnouncementResult`: struct
    * `page`: `uint32`, current page number
    * `next`: `boolean`, true if there are more results
    * `results`:


### Extrinsics
#### announce_batch(batch_announcement_params)
Creates and posts a new batch message on chain. It should create

* Parameters
  * `batch_announcement_params`: `BatchAnnouncementParams`, the parameters to use in the batch announcement.

* Event:  `Event::<T>::BatchAnnounced(schema_id file_size, file_hash, msa_id, signature)`
* Restrictions:  origin must own `msa_id`.

### Custom RPCs

#### get_batches(options)
Retrieves batch announcements that have not been garbage-collected which meet `options` criteria.

* Parameters
  1. `options`: `BatchAnnouncementOption` containing batch announcement criteria

* Return
  * `None()` if no messages meet the criteria
  * `Some([]BatchAnnouncement)`, in descending block transaction order

## Benefits and Risks
*

## Alternatives and Rationale

## Glossary
