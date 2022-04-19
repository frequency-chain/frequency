# On Chain Message Storage

## Context and Scope
The proposed feature consists of changes that is going to be one (or more) pallet(s) in runtime of a
Substrate based blockchain, and it will be used in all environments including production.

## Problem Statement
One of the core features of **MRC** is to facilitate passing messages between different
participants. To implement this core feature, backed with guarantees provided by Blockchain
technology (in our case **Substrate**) we are looking for architectures and data structures
that will allow us to store these messages on chain.

## Goals
- Allowing storage of messages with flexible schemas on chain
- Allowing high write throughput
- Allowing high read throughput
- Allowing some kind of retention mechanism to avoid infinite growth of chain database

## Proposal
Storing messages on chain using **BlockNumber** and **SchemaId** as main and secondary keys
using [StorageDoubleMap](https://docs.substrate.io/rustdocs/latest/frame_support/storage/trait.StorageDoubleMap.html) data structure provided in Substrate.

![Data-Page-3 drawio](https://raw.githubusercontent.com/Liberty30/DesignDocs/main/img/main_storage_type.png?token=GHSAT0AAAAAABPAUB3DH5DW4MIJKDCKUDQEYS57U4Q)


### Main Storage types
- **Messages**
    - _Type_: `DoubleStorageMap<BlockNumber, SchemaId, BoundedVec<Message>>`
    - _Purpose_: Main structure To store all messages for a certain block number and schema id
- **BlockMessages**
    - _Storage Type_: `StorageValue<BoundedVec<(Message, SchemaId)>>`
    - _Purpose_: A temporary storage to put each messages with it's schemaId for current block to
  increase write throughput
- **RetentionPeriods**
    - _Type_: `StorageMap<SchemaId, BlockNumber>`
    - _Purpose_: To store the retention period for each SchemaId (allows future adjustments)
    - _Defaults_: If a schema doesn't have any retention period it means there is no retention policy
  for it, and it will be remained in chain DB indefinitely.
- **StartingBlocks**
    - _Type_: `StorageMap<SchemaId, BlockNumber>`
    - _Usage_: To store the starting block number for each SchemaId (will allow future adjustments to
  `RetentionPeriods`)
    - _Defaults_: If any schemaId does not have a value inside this `StorageMap` then the Default
  starting blockNumber for it is considered as **1**.

### On Chain Structure
Following is a proposed data structure for storing a Message on chain.
```rust
pub struct Message<AccountId> {
    pub data: Vec<u8>,		    //  Serialized data in a user-defined schema format
    pub signer: AccountId,	    //  Signature of the signer
    pub msa_id: u64,                //  Message source account id (the original sender)
    pub index: u16,		    //  Stores index of message in block to keep total order
}
```

### Serialization Concerns
The initial thought around serialization was that we might want to do it on chain after validation
of the schema on each message write but due to processing restrictions on chain along with not
so good pure rust implementations support for desired serialization libraries we decided to move
schema validation and serialization processing off-chain.

Following is a list of considerations for choosing a serialization format.

1. **Message Schema Validation**
   - We should be able to validate any message against a posted schema
2. **Efficient storage**
   - On chain data storage is a limited resource, and we need to minimize stored data
3. **Supported on popular Languages**
   - To facilitate third-party integrations with **MRC** the serialization format should be
   widely supported in popular programming languages.

#### Candidates

|                  | Schema Validation<br>possibility | Efficient storage | Language<br>Support               |
|------------------|----------------------------------|-------------------|-----------------------------------|
| Json             | &#9989;                          | &#10060;          | &#9989;                           |
| Apache<br>Thrift | &#8265; Some<br>implementations  | &#9989;           | &#9989;                           |
| Protobuf         | &#9989;                          | &#9989;           | &#9989;                           |

After looking into multiple serialization formats, it appears that **Apache Thrift** and **Protobuf**
are suitable candidates. We are going to move forward with **Apache Thrift** for now since it's also
been used in other parts of the project like parquet batch files.

### Operations
#### Write
1. Off-chain: Schema of the new message will be validated against the desired schema stored on chain.
2. Off-chain: Message will be serialized using chosen serialization format mentioned above.
3. Message will be added to **BlockMessages** as a pair of `(Message, SchemaId)`
4. `on_finalize` group all Messages inside **BlockMessages** by their schemaId and add them to **Messages**
using current block number as main key and filtered schemaIds.
5. `on_finalize` set **BlockMessages** as an empty collections.
6. `on_finalize` send an `Event` for each of pair of `(currentBlock, schemaId)`

###### Side note for implementers
- Due to the limitations of `on_finalize` mentioned [here](https://docs.substrate.io/v3/runtime/benchmarking/#minimize-usage-of-on_finalize-and-transition-logic-to-on_initialize), it might be needed to do this work in the
`on_initialized` of block **n+1** rather than the `on_finalize` of block **n**.
#### Read
1. An RPC will get all messages using following params
    - `StartingBlockNumber` (inclusive)
    - `EndBlockNumber` (exclusive)
    - `schemaId`
    - `page` (starting from 0)
    - `pageSize`
2. RPC will do some initial checks on submitted params and if all are valid it will get **Messages**
from `StartingBlockNumber` until is reaches one of values from `EndBlockNumber` or `pageSize`.
3. RPC returns values using following structure
    -  content: `Vec<Message>`
    -  hasNext: `bool`
    -  nextBlock: `Optional<BlockNumber>` (has value if hasNext is true)
    -  nextPage: `Optional<u32>` (has value if hasNext is true)

#### Cleanup (Retention policy)
1. `on_initialize` remove all values from **Messages** for all blocks in following range
[`StartingBlock`, currentBlock - `StoragePeriod` - 1]
2. update `StartingBlock` to `max(StartingBlock, currentBlock - StoragePeriod - 1 ) + 1`
3. `on_initialize` calculate and return Weight for number of database read and writes for
`on_initialize` +  `on_finalize`

## Benefits and Risks
### Benefits
- Relatively high write throughput due to adding to the temporary storage called `BlockMessages`
- High read throughput for any query involving a specific block number
- Built in Support for a flexible time-based retention policy per schema
### Risks
1. Pre-defined maximum number of messages per block number enforced by [BoundedVec](https://docs.substrate.io/rustdocs/latest/frame_support/storage/bounded_vec/struct.BoundedVec.html) data type.
2. Slow read throughput for sequential data access

#### Mitigations
1. To be able to achieve high throughput we need to carefully calculate `pre-defined maximum number`
of messages per block. This number should be sufficiently big enough to satisfy **MRC**
requirements without allowing any denial of service attacks.
2. One way to improve read throughput for sequential data is to index the block numbers that have
any messages, to eliminate unnecessary DB reads. We can use a BitArray per SchemaId storing
0 if the block has no messages of that schemaId and store 1 if it does. To sustain write throughput
we would need to store this indexing data off-chain, and we can create jobs to create or update
it periodically.

![Data-OnChainAnnouncements drawio](https://raw.githubusercontent.com/Liberty30/DesignDocs/main/img/message_storage_bitvector.png?token=GHSAT0AAAAAABPAUB3DLXKOTEIG5OYRUZFAYS57YEA)
## Alternatives and Rationale
Storing messages on chain using a map of `schemaId` and `staring` index to a sequential fixed sized
bucket.

![Data-Page-2 drawio](https://raw.githubusercontent.com/Liberty30/DesignDocs/main/img/message_storage_alternative.png?token=GHSAT0AAAAAABPAUB3DTEWGIOKN5PNXSZJOYS57ZOA)

### Main Storage types
- **Messages**
    - _Type_: `DoubleStorageMap<SchemaId, Index, BoundedVec<Message>>`
    - _Purpose_: Main structure To store all messages for a certain block number and schema id
- **MessageIndices**
    - _Type_: `StorageMap<SchemaId, (u32, u32)>`
    - _Purpose_: To store current indices range for each schemaId. Tuple represents the range
  (startingIndex, endingIndex)

### Rationale
The main reason not to choose this solution is that in this architecture, writes are more expensive
compared to our proposed one and generally write throughput is more important than read.
The second drawback is that there is no direct read access from a block number to published messages
in that block without traversing through previous ones.

### Storing as Events?
- There are no fast and easy way to query and filter messages based on schemaId.
Having indexers on top of the chain will mitigate this issue but it will reduce chain self-sufficiency
- On non-archive nodes only the last **256** (by default) blocks are queryable.
This is great in terms of garbage collection but it does not provide flexibility over retention period.
## Additional Resources

* [Block Execution](https://docs.substrate.io/v3/concepts/execution/) Block execution details in Substrate.
* [Substrate Storage Items](https://docs.substrate.io/v3/runtime/storage/) Storage Items details provided by Substrate.
* [Off-Chain Features](https://docs.substrate.io/v3/concepts/off-chain-features/) Off-Chain features provided by Substrate.
