# Schemas Pallet

The Schemas Pallet provides universal schema registration for data flowing through Frequency.

## Summary

This pallet provides an on chain repository for Schemas, Intents, and IntentGroups, thereby allowing participants of the
network to flexibly interact and exchange messages with each other with the correct human intent and data structure.
All Messages and Stateful Storage content must be attached to both an Intent and a Schema Identifier so that the content
can be correctly
located, parsed, and interpreted.

## Data Structure vs Human Intent

Schemas provide for consistent data structures, while Intents encapsulate the human intent of the message.
Some schemas may be structurally the same, but have a different interpretation of the contents.
For example, two schemas might both only have a hash for contents, but one is a recording of the hash for time
validation purposes, while the other is to mark an off-chain vote.

- _Intents_ designate the semantic meaning or _purpose_ of a payload, as well as designating its _location_.
- _Schemas_ indicate how the data of a payload is to be parsed or interpreted. Many schemas can implement the same
  Intent as a data format evolves over time, but the semantic meaning or purpose does not change, nor does the storage
  lcoation.

## Groups

Intent Groups are lists of Intents that are maintained on-chain. These groups are intended mostly for off-chain
discovery by dApps to determine which Intents need to be requested as delegations for different purposes.

## Name Resolution

Intents & Intent Groups can be looked up by name (Schemas cannot). Names are of the form `<protocol>.<descriptor>`.<br/>
_NOTE: until all nodes implementing deprecated legacy RPCs are fully removed, Schemas can still be looked up by their
associated Intent
name using the `SchemasPallet_get_schema_versions_by_name` runtime API._.

## Structure

### Intent Parameters

- Settings: Various options for the Intent like signature requirements, append-only behavior, etc.
- Payload Location: Where data associated with this Intent is stored.

### Schema Parameters

- Model: The actual JSON representing the data structure.
- Model Type: The type of serialization used. (Parquet, Avro, etc...)
- Intent ID: The Intent that the Schema implements
- _NOTE: The state storage for Schemas includes a copy of the associated Intent's `settings` and `payload_location`.
  This is an optimization introduced so that Intent storage does not need to be read when writing to a Schema._

#### Model Types

- [`Parquet`](https://frequency-chain.github.io/frequency/common_primitives/schema/enum.ModelType.html#variant.Parquet):
  Designed for lists or large numbers of records; especially in the publication of record batches by a Provider on
  behalf of multiple MSAs.
- [
  `AvroBinary`](https://frequency-chain.github.io/frequency/common_primitives/schema/enum.ModelType.html#variant.AvroBinary):
  Useful for most generic data structures, preferred for on-chain data (`OnChain`, `Itemized`, and `Paginated` payload
  locations)

#### Settings

- [
  `AppendOnly`](https://frequency-chain.github.io/frequency/common_primitives/schema/enum.SchemaSetting.html#variant.AppendOnly)
    - Prior data is immutable and all new data is appended to existing data.
    - For Payload Locations: `Itemized`
- [
  `SignatureRequired`](https://frequency-chain.github.io/frequency/common_primitives/schema/enum.SchemaSetting.html#variant.SignatureRequired)
    - An MSA control key signature is required instead of a delegation.
    - For Payload Locations: `Itemized` or `Paginated`

#### Payload Locations

- [
  `OnChain`](https://frequency-chain.github.io/frequency/common_primitives/schema/enum.PayloadLocation.html#variant.OnChain):
  Data is stored directly in the Messages pallet data storage, usually as `AvroBinary`.
- [`IPFS`](https://frequency-chain.github.io/frequency/common_primitives/schema/enum.PayloadLocation.html#variant.IPFS):
  Data is stored in IPFS and Messages pallet stores the CID.
- [
  `Itemized`](https://frequency-chain.github.io/frequency/common_primitives/schema/enum.PayloadLocation.html#variant.Itemized):
  Data is stored in the Stateful Storage pallet as an array of individual items.
- [
  `Paginated`](https://frequency-chain.github.io/frequency/common_primitives/schema/enum.PayloadLocation.html#variant.Paginated):
  Data is stored in the Stateful Storage pallet as a list of paged blobs.

### Mainnet vs Testnet Entity Creation

On Mainnet, Schemas, Intents, and Intent Groups can only be created through the use of the `propose_to_create_XXX`
extrinsics, which require approval by the Frequency Council in order to be executed.
This is to prevent malicious schemas and increase the documentation around the schemas available.

On Testnets, these entities can be created by anyone using the `create_XXX` extrinsics directly, so there are _no_
guarantees around schema correctness or quality.
If you want to use a trusted Schema or Intent on a testnet, it is suggested that you use specific Schema Ids or publish
the
necessary entity yourself.

On all chains, the `create_XXX_via_governance` extrinsics exist only to support executing a proposal, and are not meant
to be invoked directly.

Note, both Testnet and local development chains are seeded with the Intents, Groups, and Schemas from Mainnet, to
facilitate ease of testing.

### Actions

The Schemas pallet provides for:

- Registering or proposing new Schemas, Intents, and Intent Groups.
- Retrieving entities by their Id (all) or name (Intents and Intent Groups).
- Retrieving last registered Schema/Intent/IntentGroup Id.
- Updating the status of a Schema
- Modifying (overwriting) the Intents contained within an IntentGroup

## Interactions

### Extrinsics

| Name/Description                                                                                                    | Caller                                          | Payment | Key Events                                                                                                                               | Runtime Added |
|---------------------------------------------------------------------------------------------------------------------|-------------------------------------------------|---------|------------------------------------------------------------------------------------------------------------------------------------------|---------------|
| `set_max_schema_model_bytes`<br />Governance action to alter the maximum byte length of Schema models               | Governance                                      | Tokens  | [`SchemaMaxSizeChanged`](https://frequency-chain.github.io/frequency/pallet_schemas/pallet/enum.Event.html#variant.SchemaMaxSizeChanged) | 1             |
| `propose_to_create_schema_v3`<br />Creates a proposal to the Frequency Council for a new schema                     | Token Account                                   | Tokens  | [`Proposed`](https://paritytech.github.io/polkadot-sdk/master/pallet_collective/pallet/enum.Event.html#variant.Proposed)                 | ?             |
| `create_schema_via_governance_v3`<br />Governance action version of `create_schema_v3`                              | Frequency Council                               | Tokens  | [`SchemaCreated`](https://frequency-chain.github.io/frequency/pallet_schemas/pallet/enum.Event.html#variant.SchemaCreated)               | ?             |
| `create_schema_v4`<br />Creates a new Schema.                                                                       | Mainnet: Governance<br />Testnet: Token Account | Tokens  | [`SchemaCreated`](https://frequency-chain.github.io/frequency/pallet_schemas/pallet/enum.Event.html#variant.SchemaCreated)               | ?             |
| `create_intent`<br />Creates a new Intent (local/testnet only)                                                      | Mainnet: filtered<br/>Testnet: Token Account    | Tokens  | `IntentCreated`                                                                                                                          | ?             |
| `create_intent_via_governance`<br/>Governance  action version of `create_intent`                                    | Frequency Council                               | Tokens  | `IntentCreated`                                                                                                                          | ?             |
| `propose_to_create_intent`<br/>Creates a proposal to the Frequency Council for a new Intent                         | Token Account                                   | Tokens  | `Proposed`                                                                                                                               | ?             |
| `create_intent_group`<br/>Creates an Intent Group                                                                   | Mainnet: prohibited<br/>Testnet: Token Account  | Tokens  | `IntentGroupCreated`                                                                                                                     | ?             |
| `create_intent_group_via_governance`<br/>Governance action version of `create_intent_group`                         | Frequency Council                               | Tokens  | `IntentGroupCreated`                                                                                                                     | ?             |
| `propose_to_create_intent_group`<br/>Creates a proposal to the Frequency Council for a new Intent Group             | Token Account                                   | Tokens  | `Proposed`                                                                                                                               | ?             |
| `update_intent_group`<br/>Overwrites an existing IntentGroup with a new list of Intents                             | Mainnet: prohibited<br/>Testnet: Token Account  | Tokens  | `IntentGroupUpdated`                                                                                                                     | ?             |
| `update_intent_group_via_governance`<br/>Governance action verison of `update_intent_group`                         | Frequency Council                               | Tokens  | `IntentGroupUpdated`                                                                                                                     | ?             |
| `propose_to_update_intent_group`<br/>Creates a proposal to the Frequency Council to update an existing Intent Group | Token Account                                   | Tokens  | `Proposed`                                                                                                                               | ?             |

See [Rust Docs](https://frequency-chain.github.io/frequency/pallet_schemas/pallet/struct.Pallet.html) for more details.

### State Queries

| Name                                    | Description                                                         | Query                                 | Runtime Added |
|-----------------------------------------|---------------------------------------------------------------------|---------------------------------------|---------------|
| Get Max Current Schema Identifier       | Fetch current Schema Identifier maximum                             | `currentSchemaIdentifierMaximum`      | 1             |
| Get Max Current Intent Identifier       | Fetch current Intent Identifier maximum                             | `currentIntentIdentifierMaximum`      | ??            |
| Get Max Current Intent Group Identifier | Fetch current IntentGroup Identifier maximum                        | `currentIntentGroupIdentifierMaximum` | ??            |
| Get Schema Model Max Bytes              | Fetch maximum number of bytes per Schema Model as set by Governance | `governanceSchemaModelMaxBytes`       | 1             |
| Get a Schema Info                       | Fetch the metadata and settings for a schema                        | `schemaInfos`                         | 62            |
| Get Schema Payload/Model                | Fetch the payload/model JSON for the specified Schema               | `schemaPayloads`                      | 62            |
| Get Intent Info                         | Fetch the metadata for an Intent                                    | `intentInfos`                         | ??            |
| Get Intent Group Info                   | Fetch the list of Intents registered in an Intent Group             | `intentGroups`                        | ??            |
| Get Intent/IntentGroup IDs by Name      | Fetch matching Intent/IntentGroup IDs by protocol and descriptor    | `nameToMappedEntityIds`               | ??            |

See the [Rust Docs](https://frequency-chain.github.io/frequency/pallet_schemas/pallet/storage_types/index.html) for
additional state queries and details.

### RPCs

Note: May be restricted based on node settings and configuration.<br/>

| Name             | Description                                  | Call                                                                                                                                    | Node Version |
|------------------|----------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------|--------------|
| Get Schema by Id | Retrieves the schema for the given Schema Id | [`getBySchemaId`](https://frequency-chain.github.io/frequency/pallet_schemas_rpc/trait.SchemasApiServer.html#tymethod.get_by_schema_id) | v1.0.0+      |

\* Must be enabled with off-chain indexing

See [Rust Docs](https://frequency-chain.github.io/frequency/pallet_schemas_rpc/trait.SchemasApiServer.html) for more
details.

### Runtime API

| Name                                                  | Description                                                    | Call                          | API Version Added | Runtime Added |
|-------------------------------------------------------|----------------------------------------------------------------|-------------------------------|-------------------|---------------|
| Get Schema by Id _(deprecated)_                       | Retrieves the schema for the given Schema Id                   | `getBySchemaId`               | 1                 | 1             |
| Get Schema by Id (version 2)                          | Retrieves the schema for the given SchemaId                    | `getSchemaById`               | 3                 | ?             |
| Get Schema Versions by Name (_deprecated_)            | Retrieves the ordered list of Schema Ids for the given name(s) | `getSchemaVersionsByName`     | 2                 | 66            |
| Get registered entities (Intent, IntentGroup) by Name | Retrieves the entities belonging to the given name(s)          | `getRegisteredEntitiesByName` | 3                 | ?             |
| Get Intent by Id                                      | Retrieves the Intent for the given IntentId                    | `getIntentById`               | 3                 | ?             |
| Get IntentGroup by Id                                 | Retrieves the IntentGroup for the given IntentGroupId          | `getIntentGroupById`          | 3                 | ?             |

See [Rust Docs](https://frequency-chain.github.io/frequency/pallet_schemas_runtime_api/trait.SchemasRuntimeApi.html) for
more details.

