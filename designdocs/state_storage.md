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

### Modes of Storage
- **Itemized**: In this mode we are storing each entity of that schemaId as an item in a blob with a
maximum size of `MAX_ITEMIZED_BLOB_SIZE`.
This is recommended for entities witch have relatively small size compared to
`MAX_ITEMIZED_BLOB_SIZE` and there are more than a few of them.
- **Paginated**: In this mode we are storing each entity individually and inside a separate page.
This mode is recommended for entities that are relatively large and there are a few of them.

![image](https://user-images.githubusercontent.com/9152501/213291600-98229ee4-6358-4e0e-abe2-d6da9abe179e.png)

The rationale behind separating these modes is based on two reasons:
- _Performance_: **Itemized** mode always needs 1 on-chain DB_READ and 1 DB_WRITE but **Paginated**
mode might be done with only 1 DB_WRITE.
- _Composability_: **Itemized** mode allows better composability due to on-chain read requirement
and smaller size of each.

### On chain Storage Details
- **Itemized**: Creating a Map of `SchemaId` as keys and `vec<u8>` as stored value containing all
items.
- **Paginated**: Creating a DoubleMap of `SchemaId` and `PageNumber` as keys and `vec<u8>` as
stored page.

To be able to achieve goal number 2 we will need to create a child-tree for each `MSA` and store
these Map and DoubleMap under these child trees for each `MSA`.

### Actions
1. **Append Item**
    - _Modes_: Itemized
    - _input_: `SchemaId` and serialized data as `vec<u8>`
    - _Purpose_: Appends a new item to existing blob
1. **Remove Item**
    - _Modes_: Itemized
    - _input_: `SchemaId` and Index of item in blob
    - _Purpose_: Removes an existing item from array
1. **Upsert**
    - _Modes_: Paginated
    - _input_: `SchemaId` and `PageNumber` and serialized data as `vec<u8>`
    - _Purpose_: Creates or updates an item
1. **RemovePage**
    - _Modes_: Paginated
    - _input_: `SchemaId` and `PageNumber`
    - _Purpose_: Removes a Page

For itemized Actions it is recommended to have a batch of actions for any schemaId to improve
performance and reduce the possibility of data races.

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
- In Itemized Storage Mode the append action will fail if we've reached `MAX_ITEMIZED_BLOB_SIZE`
size limit.
- The responsibility of data format and page management will be shifted towards offchain side which
might compromise the integrity of stored data for malicious clients.

## Using Itemized Storage for keys
To be able to add a new key we need a signature from valid control keys to ensure that this request
has end user's approval. To support this use case we will need to check if some schemas require
signature.

In general, we will need to be able to store different settings for schemas when creating and check
those settings against the actions applied to any data stored for that schema. This will allow us to
customize actions and checks that need to happen for any certain schema.
