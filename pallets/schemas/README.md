# Schemas Pallet

The Schemas Pallet provides universal schema registration for data flowing through Frequency.

## Summary

This pallet provides an on chain repository for schemas, thereby allowing participants of the network to flexibly interact and exchange messages with each other with the correct human intent and data structure.
All Messages and Stateful Storage content must be attached to a Schema Identifier so that the content can be correctly parsed and interpreted.

### Data Structure vs Human Intent

Schemas provide for both consistent data structures, but also human intent of the message.
Some schemas may be structurally the same, but have a different interpretation of the contents.
For example, two schemas might both only have a hash for contents, but one is a recording of the hash for time validation purposes, while the other is to mark an off-chain vote.

### Schema Parameters

- Model: The actual JSON representing the data structure.
- Model Type: The type of serialization used. (Parquet, Avro, etc...)
- Settings: Various options for the Schema like signature requirements.
- Payload Location: The location the data for this Schema is stored.

#### Model Types

- [`Parquet`](https://frequency-chain.github.io/frequency/common_primitives/schema/enum.ModelType.html#variant.Parquet): Designed for lists and when a Provider is collecting items from many different MSAs and publishing them together.
- [`AvroBinary`](https://frequency-chain.github.io/frequency/common_primitives/schema/enum.ModelType.html#variant.AvroBinary): Useful for most generic data structures.

#### Settings

- [`AppendOnly`](https://frequency-chain.github.io/frequency/common_primitives/schema/enum.SchemaSetting.html#variant.AppendOnly)
  - Prior data is immutable and all new data is appended to existing data.
  - For Payload Locations: `Itemized` or `Paginated`
- [`SignatureRequired`](https://frequency-chain.github.io/frequency/common_primitives/schema/enum.SchemaSetting.html#variant.SignatureRequired)
  - An MSA control key signature is required instead of a delegation.
  - For Payload Locations: `Itemized` or `Paginated`

#### Payload Locations

- [`OnChain`](https://frequency-chain.github.io/frequency/common_primitives/schema/enum.PayloadLocation.html#variant.OnChain): Data is stored directly in the Messages pallet data storage, usually as `AvroBinary`.
- [`IPFS`](https://frequency-chain.github.io/frequency/common_primitives/schema/enum.PayloadLocation.html#variant.IPFS): Data is stored in IPFS and Messages pallet stores the CID.
- [`Itemized`](https://frequency-chain.github.io/frequency/common_primitives/schema/enum.PayloadLocation.html#variant.Itemized): Data is stored in the Stateful Storage pallet as an array of individual items.
- [`Paginated`](https://frequency-chain.github.io/frequency/common_primitives/schema/enum.PayloadLocation.html#variant.Paginated): Data is stored in the Stateful Storage pallet as a list of paged blobs.

### Mainnet vs Testnet Schema Creation

Mainnet schemas must be approved by the Frequency Council.
This is to prevent malicious schemas and increase the documentation around the schemas available.

On Testnets, schemas can be created by anyone, so there are _no_ guarantees around schema correctness or quality.
If you want to use a particular schema on a testnet, it is suggested that you use specific Schema Ids or publish the needed schema yourself.

### Actions

The Schemas pallet provides for:

- Registering or proposing new Schemas.
- Retrieving schemas by their Id or name.
- Validating a Schema model.
- Retrieving last registered Schema Id.

## Interactions

### Extrinsics

| Name/Description                                                                                      | Caller                                          | Payment | Key Events                                                                                                                               | Runtime Added |
| ----------------------------------------------------------------------------------------------------- | ----------------------------------------------- | ------- | ---------------------------------------------------------------------------------------------------------------------------------------- | ------------- |
| `set_max_schema_model_bytes`<br />Governance action to alter the maximum byte length of Schema models | Governance                                      | Tokens  | [`SchemaMaxSizeChanged`](https://frequency-chain.github.io/frequency/pallet_schemas/pallet/enum.Event.html#variant.SchemaMaxSizeChanged) | 1             |
| `propose_to_create_schema_v2`<br />Creates a proposal to the Frequency Council for a new schema       | Token Account                                   | Tokens  | [`Proposed`](https://paritytech.github.io/polkadot-sdk/master/pallet_collective/pallet/enum.Event.html#variant.Proposed)                 | 66            |
| `create_schema_via_governance_v2`<br />Governance action version of `create_schema_v3`                | Frequency Council                               | Tokens  | [`SchemaCreated`](https://frequency-chain.github.io/frequency/pallet_schemas/pallet/enum.Event.html#variant.SchemaCreated)               | 66            |
| `create_schema_v3`<br />Creates a new Schema.                                                         | Mainnet: Governance<br />Testnet: Token Account | Tokens  | [`SchemaCreated`](https://frequency-chain.github.io/frequency/pallet_schemas/pallet/enum.Event.html#variant.SchemaCreated)               | 1             |
| `propose_to_create_schema_name`<br />Creates a Council proposal to set the name of a Schema           | Token Account                                   | Tokens  | [`Proposed`](https://paritytech.github.io/polkadot-sdk/master/pallet_collective/pallet/enum.Event.html#variant.Proposed)                 | 1             |
| `create_schema_name_via_governance`<br />Governance action to set the name of a Schema                | Frequency Council                               | Tokens  | [`SchemaNameCreated`](https://frequency-chain.github.io/frequency/pallet_schemas/pallet/enum.Event.html#variant.SchemaNameCreated)       | 66            |

See [Rust Docs](https://frequency-chain.github.io/frequency/pallet_schemas/pallet/struct.Pallet.html) for more details.

### State Queries

| Name                              | Description                                                         | Query                            | Runtime Added |
| --------------------------------- | ------------------------------------------------------------------- | -------------------------------- | ------------- |
| Get Max Current Schema Identifier | Fetch current Schema Identifier maximum                             | `currentSchemaIdentifierMaximum` | 1             |
| Get Schema Model Max Bytes        | Fetch maximum number of bytes per Schema Model as set by Governance | `governanceSchemaModelMaxBytes`  | 1             |
| Get a Schema Info                 | Fetch the metadata and settings for a schema                        | `schemaInfos`                    | 62            |
| Get Schema Ids by Name            | Fetch matching Schemas Ids by namespace and name                    | `schemaNameToIds`                | 62            |
| Get Schema Payload/Model          | Fetch the payload/model JSON for the specified Schema               | `schemaPayloads`                 | 62            |

See the [Rust Docs](https://frequency-chain.github.io/frequency/pallet_schemas/pallet/storage_types/index.html) for additional state queries and details.

### RPCs

Note: May be restricted based on node settings and configuration.

| Name                  | Description                                                         | Call                                                                                                                                               | Node Version |
| --------------------- | ------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------- | ------------ |
| Get Schema by Id      | Retrieves the schema for the given Schema Id                        | [`getBySchemaId`](https://frequency-chain.github.io/frequency/pallet_schemas_rpc/trait.SchemasApiServer.html#tymethod.get_by_schema_id)            | v1.0.0+      |
| Check Schema Validity | Validates a schema model and returns “true” if the model is correct | [`checkSchemaValidity`](https://frequency-chain.github.io/frequency/pallet_schemas_rpc/trait.SchemasApiServer.html#tymethod.check_schema_validity) | v1.0.0+      |
| Get Schema Versions   | Returns an array of schema versions                                 | [`getVersions`](https://frequency-chain.github.io/frequency/pallet_schemas_rpc/trait.SchemasApiServer.html#tymethod.get_versions)                  | v1.10.0+     |

\* Must be enabled with off-chain indexing

See [Rust Docs](https://frequency-chain.github.io/frequency/pallet_schemas_rpc/trait.SchemasApiServer.html) for more details.
