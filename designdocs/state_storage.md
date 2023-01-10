# On Chain State Storage

## Context and Scope
The proposed feature consists of changes that is going to be one (or more) pallet(s) in runtime of a
Substrate based blockchain, and it will be used in all environments including production.

## Problem Statement
Every party using any medium of communication will need to deal with the storage of some kind of
stateful data. Some of these stateful data might be consensus critical and in a decentralized world
one option might be to store this type of data on a blockchain as long as it is required.
The scope for how long some of these stateful data should exist is dependent on the
type of data. For example, some of them might exist as long as an active channel exists between two
communicating parties and for some others it might be directly dependent on the existence of a
particular party.

### DSNP Usecase
**Frequency** is the first implementation of [DSNP](https://dsnp.org) and most of the state
transitions in **DSNP** can be modeled via `Announcements`. For some of these state transitions we
only care about the latest state, and currently the only way to achieve this via announcements is to
have some kind of third party indexer to track the latest state for these types.
If there was a way to be able to mutate the state of a storage, then we could update it to its
latest version and potentially remove the need for an indexer.

## Goals
1. Allowing storage of stateful data with flexible schemas on chain.
2. Data stored for any `MSA` should not have any effects like changing read/write access times or
storage costs for any data stored for another `MSA`.
3. Allowing high write throughput.
4. Allowing high read throughput.
5. Avoiding data races between consecutive updates of the same item.

## Non-goals
- Defining an economic incentive to restrict infinite growth of on-chain database.

## Glossary
* `DSNP`: Decentralized Social Networking Protocol
* `MSA`: Message Source Account. A registered identifier with the MSA pallet.

## Proposal
Creating a double map of (`SchemaId` and `PageNumber`) as keys and `vec<u8>` as stored values.

Using mentioned double-map will allow storing data using a variety of schemas using pagination.To
be able to achieve goal number 2 we will need to create a child-tree for each `MSA` and store this
double map under these child trees for each `MSA`.

### Modes of Storage
- **Batch**: In this mode we are storing each entity of that schemaId as an item inside an array.
This is recommended for items that are relatively small and there are more than a few of them.
- **Single**: In this mode we are storing each entity individually, and it is recommended for items
that are relatively large and there are a few of them.

![image](https://user-images.githubusercontent.com/9152501/211664660-1d4aa9ac-ed11-44eb-a8f4-66c972750ac3.jpg)

The rationale behind separating these modes is based on two reasons:
- _Performance_: Batch modes always need 1 on-chain DB_READ and 1 DB_WRITE but Single mode might be
done with only 1 DB_WRITE.
- _Composability_: Batch mode allows better composability due to on-chain read requirement and smaller
size of each item.

### Actions
1. **Append Item**
    - _Modes_: Batch
    - _input_: Serialized data `vec<u8>`
    - _Purpose_: Adds a new item to existing array
1. **Remove Item**
    - _Modes_: Batch
    - _input_: Index of item in array
    - _Purpose_: Removes an existing item from array
1. **Replace Item**
    - _Modes_: Batch
    - _input_: index of item to replace and `vec<u8>` of new data
    - _Purpose_: Updates an item in the array
1. **Upsert**
    - _Modes_: Single
    - _input_: Serialized data `vec<u8>`
    - _Purpose_: Creates or updates an item
1. **Remove**
    - _Modes_: Single, Batch
    - _Purpose_: Removes a Single or Batch node.

### Pre Checks
1. Checking schema (id and structure) against submitted data
2. Checking schema permission and grants
3. Checking Hash of previous state to avoid data races

To be able to achieve all mentioned goals we will need to be able to do all or most of the mentioned
checks offchain. (Permission grant check will need to be on-chain due to importance!)

## Benefits and Risks
### Benefits
- Highly available and consistent and decentralized data storage
- `MSA` based data isolation
- Data format flexibility using different schemas
- Providing a protocol and usage agnostic consensus critical storage
### Risks
- Requires well-defined economical incentives to minimize the storage size and time to allow
scalability
- A mechanism to prune data for non-active users (maybe migrate data to IPFS or S3 for long term
storage)
- The responsibility of data format and page management will be shifted towards offchain side which
might compromise the integrity of stored data for malicious clients.
