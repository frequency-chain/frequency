# On Chain Message Storage

## Context and Scope
The proposed feature consists of changes that is going to be one (or more) pallet(s) in runtime of a
Substrate based blockchain, and it will be used in all environments including production.

## Problem Statement
After introduction of **Proof of Validity** or **PoV** in runtime weights, all pallets should be
re-evaluated and refactored if necessary to minimize the usage of **PoV**. This is to ensure all
important operations are scalable.
This document tries to propose some changes on **Messages** pallet to optimize the **PoV** size.

## Goals
- Minimizing Weights including **execution times** and **PoV** size.

## Proposal
Storing messages on chain using **BlockNumber** and **SchemaId** and **MessageIndex** as main and secondary
 and tertiary keys using [StorageNMap](https://paritytech.github.io/substrate/master/frame_support/storage/trait.StorageNMap.html) data structure provided in Substrate.

### Main Storage types
- **MessagesV2**
    - _Type_: `StorageNMap<(BlockNumber, SchemaId, MessageIndex), Message>`
    - _Purpose_: Main structure To store all messages for a certain block number and schema id and
      index


### On Chain Structure
Following is a proposed data structure for storing a Message on chain.
```rust
/// only `index` is removed from old structure
pub struct Message<AccountId> {
    pub payload: Vec<u8>,		    //  Serialized data in a user-defined schemas format
    pub provider_key: AccountId,	    //  Signature of the signer
    pub msa_id: u64,                //  Message source account id (the original source of the message)
}
```
## Description

The idea is to use existing **whitelisted** storage with `BlockMessageIndex` type to store and get
the index of each message to be able to use it as our third key for `StorageNMap`.

We would store each message separately into `StorageNMap` with following keys
- primary key would be `block_number`
- secondary key would be `schema_id`
- tertiary key would be the `index` of the message for current block which starts from 0


