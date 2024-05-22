# Messages Pallet

Stores block-indexed data for a Schema using `OnChain` or `IPFS` payload location.

## Summary

Messages stores metadata and message payload data for a set Schema.
Use of the Message pallet payloads is designed for time-centric data.
Discovery is via the `MessagesInBlock` on new blocks or requesting messages from a block.
Retrieval is via RPC.

### Metadata vs Payload

Messages have both metadata and payloads.
The payload should always match the data structure or the messages is considered invalid.
The metadata is the Block Number, Schema Id, and other data to help discover and organize the payload information.

### Payload Options

- `IPFS`: Storage of the CID and length of the file on IFPS
- `OnChain`: Storage of the entire payload data. Usually for tiny payloads only.

### Message Ordered

Messages are ordered by block number, but the have a set order based on transaction order in the block.
This order is immutable.

### Actions

The Messages pallet provides for:

- Adding messages for a given schema
- Enabling the retrieval of messages for a given schema

## Interactions

### Extrinsics

| Name/Description                                                                         | Caller   | Payment            | Key Events                                                                                                                        | Runtime Added |
| ---------------------------------------------------------------------------------------- | -------- | ------------------ | --------------------------------------------------------------------------------------------------------------------------------- | ------------- |
| `add_ipfs_message`<br />Add a message to a Schema with an `IPFS` payload location        | Provider | Capacity or Tokens | [`MessagesInBlock`](https://frequency-chain.github.io/frequency/pallet_messages/pallet/enum.Event.html#variant.MessagesInBlock)\* | 1             |
| `add_onchain_message`<br />Add a message to a Schema with an `ON_CHAIN` payload location | Provider | Capacity or Tokens | [`MessagesInBlock`](https://frequency-chain.github.io/frequency/pallet_messages/pallet/enum.Event.html#variant.MessagesInBlock)\* | 1             |

\* The `MessagesInBlock` may not occur more than once per block and does _not_ indicate which schema received messages.

See [Rust Docs](https://frequency-chain.github.io/frequency/pallet_messages/pallet/struct.Pallet.html) for more details.

### State Queries

| Name            | Description                                                                                                                   | Query        | Runtime Added |
| --------------- | ----------------------------------------------------------------------------------------------------------------------------- | ------------ | ------------- |
| Get Messages v2 | _Suggested_: Use RPC instead of this storage directly. Storage for the messages by Block number, Schema Id, and Message Index | `messagesV2` | 61            |
| Get Messages v1 | Removed in Runtime 60                                                                                                         | `messages`   | 1-60          |

See the [Rust Docs](https://frequency-chain.github.io/frequency/pallet_messages/pallet/storage_types/index.html) for additional state queries and details.

### RPCs

Note: May be restricted based on node settings and configuration.

| Name                      | Description                                                                                      | Call                                                                                                                                               | Node Version |
| ------------------------- | ------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------- | ------------ |
| Get Messages by Schema Id | Fetch paginated messages for a specific Schema Id in the given block range for a given Schema Id | [`getBySchemaId`](https://frequency-chain.github.io/frequency/pallet_messages_rpc/trait.MessagesApiServer.html#tymethod.get_messages_by_schema_id) | v1.0.0+      |

See [Rust Docs](https://frequency-chain.github.io/frequency/pallet_messages_rpc/trait.MessagesApiServer.html) for more details.
