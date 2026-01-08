# Messages Pallet

Stores block-indexed data for a Schema using `OnChain` or `IPFS` payload location.

## Summary

`messages` stores metadata and message payload data for a set Schema.
Message payloads are meant for streaming data, where _when_ the message was sent is the focus.
Discovery is via the `MessagesInBlock` on new blocks or requesting messages from a block.
Retrieval is via a custom runtime API.

### Metadata vs Payload

Messages have both metadata and payloads.
The payload should always match the data structure or the message is considered invalid.
The metadata is the Block Number, Intent Id, and other data useful for discovering and organizing the payload information.

### Payload Options

- `IPFS`: Storage of the CID and length of the file on IPFS
- `OnChain`: Storage of the entire payload data, usually for sub-256 byte payloads

### Message Ordering

Messages are ordered by block number and IntentId, and within each block, they follow a specific order based on their transaction sequence within that block.
This order is immutable.

### Actions

The Messages pallet provides for:

- Adding messages for a given Intent
- Enabling the retrieval of messages for a given Intent

## Interactions

### Extrinsics

| Name/Description                                                                         | Caller   | Payment            | Key Events                                                                                                                        | Runtime Added |
|------------------------------------------------------------------------------------------|----------|--------------------|-----------------------------------------------------------------------------------------------------------------------------------|---------------|
| `add_ipfs_message`<br />Add a message to a Schema with an `IPFS` payload location        | Provider | Capacity or Tokens | [`MessagesInBlock`](https://frequency-chain.github.io/frequency/pallet_messages/pallet/enum.Event.html#variant.MessagesInBlock)\* | 1             |
| `add_onchain_message`<br />Add a message to a Schema with an `ON_CHAIN` payload location | Provider | Capacity or Tokens | [`MessagesInBlock`](https://frequency-chain.github.io/frequency/pallet_messages/pallet/enum.Event.html#variant.MessagesInBlock)\* | 1             |

\* The `MessagesInBlock` may occur at most once per block and does _not_ indicate which Intent(s) received messages.

See [Rust Docs](https://frequency-chain.github.io/frequency/pallet_messages/pallet/struct.Pallet.html) for more details.

### State Queries

| Name       | Description                                                                                                                                           | Query        | Runtime Added |
|------------|-------------------------------------------------------------------------------------------------------------------------------------------------------|--------------|---------------|
| MessagesV3 | Suggested: Use custom runtime API instead of querying this storage directly.<br/>Storage for the messages by Block Number, IntentId, and MessageIndex | `messagesV3` | 184           |
| MessagesV2 | Removed in Runtime 184                                                                                                                                | `messagesV2` | 61            |
| Messages   | Removed in Runtime 60                                                                                                                                 | `messages`   | 1-60          |

See the [Rust Docs](https://frequency-chain.github.io/frequency/pallet_messages/pallet/storage_types/index.html) for additional state queries and details.

### RPCs

Note: May be restricted based on node settings and configuration.

| Name                                     | Description                                                                                                                                                                             | Call                                                                                                                                               | Node Version |
|------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------|--------------|
| Get Messages by Schema Id _(deprecated)_ | Fetch paginated messages for a specific Schema Id in the given block range for a given Schema Id<br/>Deprecated in `v2.0.0`. Use custom Runtime API `get_messages_by_intent_id` instead | [`getBySchemaId`](https://frequency-chain.github.io/frequency/pallet_messages_rpc/trait.MessagesApiServer.html#tymethod.get_messages_by_schema_id) | v1.0.0+      |

See [Rust Docs](https://frequency-chain.github.io/frequency/pallet_messages_rpc/trait.MessagesApiServer.html) for more details.

### Runtime API

| Name                                            | Description                                                               | Call                          | API Version Added | Runtime Added |
|-------------------------------------------------|---------------------------------------------------------------------------|-------------------------------|-------------------|---------------|
| Get Schema by Id _(deprecated)_                 | Retrieves the schema for the given Schema Id                              | `getBySchemaId`               | 1                 | 1             |
| Get Messages by Schema and Block _(deprecated)_ | Retrieve the messages for a particular schema and block number            | `getMessagesBySchemaAndBlock` | 1                 | 1             |
| Get Messages by Intent and Block                | Retrieve the messages for a particular intent and block range (paginated) | `getMessagesByIntentId`       | 2                 | 184           |

See [Rust Docs](https://frequency-chain.github.io/frequency/pallet_messages_runtime_api/trait.MessagesRuntimeApi.html) for
more details.
