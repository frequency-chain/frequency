# 📘 Design Discussion: Schema, Protocols, and Intent-Based Delegation in Frequency

## 1. **Background and Motivation** <a id="section_1"></a>

In the current implementation, schemas are registered with immutable numeric identifiers (`SchemaId`) and describe the
layout and storage semantics (e.g., Avro/Parquet formats, on-chain/off-chain storage). These schema IDs are used as
references by clients and runtime modules alike, particularly in the delegation system defined by the `msa` pallet.

Delegations currently allow a user to authorize a provider (e.g., an app or service) to act on their behalf, but this
authorization is tightly bound to a specific `SchemaId`. This model has proven limiting in several ways:

- **Data format evolution requires re-delegation (expensive and cumbersome)**
- **Schemas represent data format, not purpose**
- **Lack of human-readable context**

Additionally, delegation housekeeping is cumbersome for Providers, due to the need to delegate multiple schemas that
form a functional group; the determination of which schemas to delegate cannot be discovered except by a manual reading
of separate documentation.

These limitations have motivated a re-architecture of the schema and delegation systems to introduce the concepts of:

- **Named intents** with version tracking
- **Intent-based delegation**
- **More flexible storage models**
- **Named intent groups** that facilitate functional delegation and discovery

## 2. **Design Goals** <a id="section_2"></a>

This section outlines the key objectives that guide the redesign of Frequency's schema and delegation architecture.

- **Schema Immutability** - Individual schema versions, once published, are immutable on-chain
- **Minimal Delegation Churn** - Minor changes to data formats should not require new delegations
- **Minimal Storage Churn (migrations)** - Changes to storage formats should not require mass migration of user
  data
- **Intent Separation** - When permissioning, we need to be able to separate the purpose of the data and action from its
  format
- **On-Chain Efficiency** - On-chain operations need to be efficient, so storage and structures must be designed with
  that in mind

## 3. **Current Design** <a id="current_design"></a>

```mermaid
erDiagram
    SchemaNameToIds {
        string schema_namespace PK
        string schema_descriptor PK
    }
    SchemaVersionId {
        integer[] schema_versions
    }
    Delegation {
        integer msa_id PK
        integer provider_id PK
        integer revoked_at
    }
    DelegationSchemaPermissions {
        integer schema_id PK
        integer revoked_at
    }
    Schema {
        integer id PK
        data SchemaInfo
    }

    SchemaNameToIds ||--|| SchemaVersionId: "name-to-id mapping"
    SchemaVersionId ||--o{ Schema: "schema version mapping"
    Delegation ||--o{ DelegationSchemaPermissions: "has permissions"
    DelegationSchemaPermissions ||--o{ Schema: "delegation mapping"
```

## 4.**Proposed Design Diagram: Immutable, Versioned Schemas with Intents**<a id="proposed_design"></a>

NOTE: For simplicity, I've omitted showing entities/relations whose sole purpose is providing name-to-id lookup for
off-chain clients

```mermaid
---
title: "Entities & Relationships"
---
erDiagram
    Intent {
        integer id PK
        integer[] schema_versions FK
    }
    Delegation {
        integer msa_id PK
        integer provider_id PK
        integer revoked_at
    }
    DelegationIntentPermissions {
        integer intent_id FK
        integer revoked_at
    }
    IntentGroup {
        integer id PK
        integer[] intents FK
    }
    Schema {
        integer id PK
        integer intent_id FK
        data schema_metadata
        data schema_payload
    }

    Intent ||--o{ Schema: "intent-to-schemas"
    Delegation ||--o{ DelegationIntentPermissions: ""
    DelegationIntentPermissions ||--|| Intent: ""
    IntentGroup }o--o{ Intent: "intent-group-to-intents"
```

### Notes

- `Intents` MUST be mutable; otherwise there's little benefit to them (the main benefit of an Intent being that it
  enables mutating the collection of permissions without requiring a new delegation)
- `Schemas` are NOT mutable; they represent a fixed format & payload location
- The bi-directional lookup on `Intent` <--> `Schema` is crucial to mitigating the runtime cost of delegation lookups
- The cost of doing a Delegation lookup for a particular Schema is the same as the current implementation
- `IntentGroups` are _mutable_--but, critically, are not themselves delegatable. That is, granting delegations by
  IntentGroup merely creates the individual Intent delegations that exist in the group _at the time of delegation_;
  subsequent mutations of the IntentGroup do not affect existing delegations. Granting delegations in this way may
  be supported by new extrinsics, or may simply be left to the client to query the IntentGroup and request the
  indicated delegations.
- Because stored data retains an indication of the concrete `SchemaId` that was used to write it, there is ZERO risk of
  introducing a breaking format change, as users will always have access to the correct schema needed to decode the
  data.

```mermaid
---
title: "Pallet Storage Model"
---
erDiagram
    Messages {
        integer block_number PK
        integer intent_id PK
        integer message_index PK
    }

    MessagePayload {
        integer schema_id
        data payload
    }

    StatefulStorage {
        integer msa_id PK
        integer intent_id PK
        integer page_index PK
    }

    StatefulStoragePage {
        integer page_nonce
        integer schema_id
        data payload
    }

    Messages }o--o{ MessagePayload: "messages in block"
    StatefulStorage }o--o{ StatefulStoragePage: "storage trie pages"
```

### Notes

This design separates the notion of _storage location_ from _data format_ (ie, `Schema`). _Storage location_ is now tied
to `IntentId`.

The design requires modifications to the pallet storage structures for both the `messages` and `stateful-storage`
pallets. While this could be accomplished via a migration of all existing pallet data, the amount of data that currently
exists on-chain makes this problematic. If the cost or complexity of such a migration renders it infeasible, the
following approach is proposed:

#### `messages` pallet

Since `messages` pallet storage represents time-series content publications, it should be possible to define a
`MessagesV3` pallet storage (the current storage being `MessagesV2`). All future write operations would write to
`MessagesV3`. For reads, we would store the block number at which `MessagesV3` was introduced; read requests for data
prior to that block would read from `MessagesV2`.

#### `stateful-storage` pallet

Data stored in the `stateful-storage` pallet always represents the latest state, rather than a time-series. Therefore,
it's difficult or impossible to bifurcate the storage in the same way as the `messages` pallet. Instead, to avoid
requiring a complete storage migration, new pages/items that are written can include a _storage version magic number_ in
either the page or the item header. For `Paginated` storage, this value would precede the `PageNonce`; for `Itemized`
storage the value would precede `payload_len`. The 'magic number' would be designed to be the same byte length as the
value currently at byte offset zero within the page/item, and to be a value such that conflict with a valid `nonce` or
`payload_len` would be highly unlikely, if not impossible.

New structures would be defined, ie `PageV2` and `ItemizedItemV2`, and decoding values read from storage would need to
determine which structure to decode to based on the presence/absence of the "magic value".

## 5. **Delegation Semantics**<a id="delegation_semantics"></a>

Delegation is the mechanism by which a user authorizes a provider to act on their behalf. Currently, this is limited to
individual `SchemaId`s, but we propose changing the delegation model to be based on `IntentId`. The structure of a
Delegation would not change; an initial migration would create a single `Intent` for each existing `Schema`, with the
same numeric ID, so that current Delegation storage would not require a migration.

## 6. **Schemas**<a id="schemas"></a>

In the new model, a `Schema` represents a data format definition *only*. Any association with data _meaning_ or _storage
location_ is promoted to the Schema's corresponding `Intent`. A Schema may be associated with one and only one Intent.
Under this model, a Schema has some associated metadata, and a model containing the actual data format definition (ie,
currently-supported Parquet or Avro schema).

### **Schema Versioning**<a id="schema_versioning"></a>

Schemas may be loosely "versioned" in the sense that they may be marked as deprecated or unsupported. A status of
`Deprecated` is merely advisory for a client application to be aware of, while a status of `Unsupported` prohibits
write operations on-chain for that schema. Updates to a Schema's status will be implemented as separate governance-
controlled operations.

With the exception of the `status` field, Schemas are otherwise immutable once published.
The associated data types would be as follows:

```rust
pub type SchemaId = u16;

pub type SchemaModel = BoundedVec<u8, T::SchemaModelMaxBytesBoundedVecLimit>;

pub enum SchemaStatus {
    /// Schema is current and approved for writing
    Active,
    /// Schema is viable for writing, but is deprecated and may become unsupported soon
    Deprecated,
    /// Schema is unsupported; writing to this schema is prohibited
    Unsupported,
}

pub struct SchemaInfo {
    /// The type of model (AvroBinary, Parquet, etc.)
    pub model_type: ModelType,
    /// The associated Intent
    pub intent_id: IntentId,
    /// Status of the Schema
    pub status: SchemaStatus,
}
```

## 7. **Intents**<a id="intents"></a>

`Intents` represent a consistent _data meaning or purpose_ for which a user may delegate permission. Intents may be
associated with one or more Schemas, which comprise a _version list_. Via this mechanism, Intents are able to represent
the evolution of a data format. An Intent may also be associated with no schemas at all; this allows us to create
delegatable permissions or entitlements on-chain that may be checked by on- or off-chain applications.

Intents are mutable, but only in the sense that publication of new Schemas representing the same evolving data format
may be associated with the same Intent. Intents are immutable once published.

When associated a `payload_location`, Intents also represent a _storage location_ and other attributes. This
approach allows on-chain data to evolve over time. Since the storage location of an Intent remains constant, and data is
written with an indication of the specific SchemaId used to encode it, publication of a new Schema does not require
wholesale data migration. Instead, on-chain data may be migrated by Provider applications opportunistically over time.
Off-chain data may persist in its existing form and can always be read/decoded using the original Schema definition
used to write it. Intents with a `payload_location` of `None` are considered "schemaless" Intents designated for
off-chain
interpretation.

The structures and types for Intents are envisioned as follows:
<a id="intent_struct"></a>

```rust
pub type IntentId = u16;

// Renamed from existing `SchemaSetting`
pub enum IntentSetting {
    /// Intent setting to enforce append-only behavior on payload.
    /// Applied to Intents with `payload_location: PayloadLocation::Itemized`.
    AppendOnly,
    /// Intent may enforce a signature requirement on payload.
    /// Applied to intents with `payload_location: PayloadLocation::Itemized` or `PayloadLocation::Paginated`.
    SignatureRequired,
}

pub struct IntentSettings(pub BitFlags<IntentSetting>);

pub struct IntentInfo {
    /// The payload location
    pub payload_location: Option<PayloadLocation>,
    /// additional control settings for the schema
    pub settings: IntentSettings,
}
```

### 8. **Intent Groups**<a id="intent_groups"></a>

#### Rationale

In order to support all desired operations for a particular function, it is often necessary for a Provider to obtain
multiple, related delegations. The original design allowed for no discovery mechanism, leaving it as an exercise for the
developer to discover through external documentation. Furthermore, there was no mechanism to become aware when the
required list of delegations changed.

#### Details

As mentioned <a href="#delegation_semantics">above</a>, other than changing the interpretation of a Delegation from
`SchemaId` to `IntentId`, the semantics of Delegations does not change in the new design. However, to
facilitate user provisioning and onboarding by Providers, we introduce here the concept of _Intent Groups_.

A `IntentGroup` is a list of `IntentIds` that are "bundled" together. These bundles may be resolved to the discrete
contained `IntentIds` when a Provider seeks to request or verify delegations for a common purpose.

Intent Groups **must be resolved to individual `IntentIds` at the time of delegation granting**. In this sense, they
are both mutable _and_ immutable:

* _immutable_ in the sense that when granting delegations based on an Intent Group, the list of Intents so delegated
  may not be changed without another explicit delegation action by the user.
* _mutable_ in the sense that the list of Intents associated with an Intent Group may change over time, providing a
  mechanism to check that a user has all the necessary or desired delegations in place.

This model preserves Frequency's user-security model of _explicit delegation_, while simultaneously gives Providers a
convenience mechanism for evolving sets of permissions.

The structure for Intent Groups is proposed as follows:

<a id="intent_group_struct"></a>

```rust
pub type IntentGroupId = u16;

pub struct IntentGroup {
    /// List of Intents associated with this Delegation
    pub intent_ids: BoundedVec<IntentId, ConstU32<MAX_INTENTS_PER_GROUP>>,
}
```

#### Example

Consider this example: imagine a fictitious function "have a party". This requires the following Intent delegations:

* `party.eat` (intent_id: 1)
* `party.drink` (intent_id: 2)

In order to facilitate discovery of this group of Intents, we create the following `IntentGroup`:

* `party.requirements` [1, 2]

We can discover from the chain itself the list of Intents that need to be delegated, from a single reference (the
IntentGroup name, 'party.requirements').

Now, imagine the list of required Intents changes; we now require an additional delegation:

* `party.be_merry` (intent_id: 12)

We can update the IntentGroup as follows:

* `party.requirements` [1, 2, 12]

Now, we can compare the existing delegations for users on-chain with the set of required delegations and activate an
appropriate workflow to add any missing delegations. Previously, there was no way to know that this list of required
delegations had changed. The developer would simply have to monitor a documentation site for (hopefully timely) updates,
then make the appropriate changes to a client app to add the new Intent to the list of required delegations.

### 9. **Name Resolution**<a id="name_resolution"></a>

In addition to the new & updated primitives for Schemas, Intents, and Intent Groups, this design also provides for a
name resolution mechanism so that off-chain applications may discover the necessary on-chain identifiers. These
facilities are _solely for off-chain name resolution_; all on-chain extrinsics and other calls will require the
appropriate numeric identifier (i.e., `SchemaId`, `IntentId`, `IntentGroupId`).

Related names will be grouped under a top-level identifier called a 'protocol'. This enables querying the chain by a
fully qualified name `<protocol>.<name>`, or by `<protocol>` only for a list of registered names and their corresponding
entities. For example, using data currently on Frequency Mainnet, we would have two protocols defined: 'dsnp' and '
bsky'.
Each name registered to a protocol points to either an `IntentId` or a `IntentGroupId`. The structures for the name
registry would look as follows:

<a id="name_registry_struct"></a>

```rust
pub enum RegisteredNameIdType {
    Intent(IntentId),
    IntentGroup(IntentGroupId),
}

/// Protocol name type
pub type ProtocolName = BoundedVec<u8, ConstU32<PROTOCOL_MAX>>;
/// descriptor type
pub type NameDescriptor = BoundedVec<u8, ConstU32<DESCRIPTOR_MAX>>;
```

**NOTE:** The '\<protocol>.\<name>' mapping and registry may be replaced at some future date if Frequency implements a
true ENS registry.

### 10. Ownership and Governance<a id="ownership_governance"></a>

It may be desirable at some point to implement the concept of ownership of protocols and the entities & names registered
under them, thereby enabling the concept of publishing authority for the creation of new Intents, Schemas, and
Intent Groups. However, that is considered out of scope of the current design. It may be evaluated at a later date,
possibly in the context of a full DAO implementation for Frequency.

Instead, for the proposed design, as with the current design, all additions & changes to Schemas, Intents, Intent
Groups, and Name registrations must be approved by Governance. Specifically, the following actions must be
Governance-approved:

| Action                                                        | Considerations                                                                           |
|---------------------------------------------------------------|------------------------------------------------------------------------------------------|
| Publish a new Schema                                          | Is the Schema an evolution of existing Schemas registered to the Intent?                 |
| Publish a new (named) Intent                                  | Does the requestor represent an org with authority to publish to the indicated protocol? |
| Publish a new (named) Intent Group<br/>Update an Intent Group | Does the requestor represent an org with authority to publish to the indicated protocol? |

### 11. **Extrinsics**<a id="extrinsics"></a>

The following modifications to existing extrinsics are proposed:

* `propose_to_create_schema_v2`, `create_schema_via_governance_v2`, and `create_schema_v3` (deprecated)
    * Will reject if `schema_name` is `None`.
    * Will reject if `schema_name` does not resolve to an existing Intent.
    * Will reject if the supplied `payload_location` or `settings` do not match the associated Intent's values
* `propose_to_create_schema_name` and `create_schema_name_via_governance` (deprecated)
    * No-op, as schema names are no longer supported
* All extrinsics related to Delegations that reference `SchemaId` will be changed to reference `IntentId`. Since the
  data type is `u16` for both, the binary API will not change.

The following new extrinsics are proposed:

| Extrinsic                                                | Parameters                                                                                                                                                          | Description                                                            |
|----------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------|------------------------------------------------------------------------|
| propose_to_create_schema_v3<br/>create_schema_v4         | `model: BoundedVec<u8>`<br/>`model_type: ModelType`<br/>`intent_id: IntentId`                                                                                       | Propose to create a Schema<br/>Create a Schema                         |
| create_schema_via_governance_v3                          | `creator_key: AccountId`<br/>`model: BoundedVec<u8>`<br/>`model_type: ModelType`<br/>`intent_id: IntentId`                                                          | Create a Schema via governance                                         |
| propose_to_create_intent<br/>create_intent               | `protocol_name: ProtocolName`<br/>`intent_name: NameDescriptor`<br/>`payload_location: PayloadLocation`<br/>`settings: IntentSettings`                              | Propose to create an Intent<br/>Create an Intent                       |
| create_intent_via_governance                             | `creator_key: AccountId`<br/>`protocol_name: ProtocolName`<br/>`intent_name: NameDescriptor`<br/>`payload_location: PayloadLocation`<br/>`settings: IntentSettings` | Create an Intent via Governance                                        |
| propose_to_create_intent_group<br/>create_intent_group   | `protocol_name: ProtocolName`<br/>`group_name: NameDescriptor`<br/>`intent_ids: BoundedVec<IntentId, MAX_INTENTS_PER_GROUP>`                                        | Propose to create a IntentGroup<br/>Create an IntentGroup              |
| create_intent_group_via_governance                       | `creator_key: AccountId`<br/>`protocol_name: ProtocolName`<br/>`group_name: NameDescriptor`<br/>`intent_ids: BoundedVec<IntentId, MAX_INTENTS_PER_GROUP>`           | Create an IntentGroup via Governance                                   |
| propose_to_update_intent_group<br/>update_intent_group   | `group_id: IntentGroupid`<br/>`intent_ids: BoundedVec<IntentId, MAX_INTENTS_PER_GROUP>`                                                                             | Propose to update an IntentGroup<br/>Update (overwrite) an IntentGroup |
| update_intent_group_via_governance                       | `creator_key: AccountId`<br/>`group_id: IntentGroupid`<br/>`intent_ids: BoundedVec<IntentId, MAX_INTENTS_PER_GROUP>`                                                | Update (overwrite) an IntentGroup via Governance                       |
| propose_to_update_schema_status<br/>update_schema_status | `schema_id: SchemaId`<br/>`status: SchemaStatus`                                                                                                                    | Propose to update a Schema's status<br/>Update a Schema's status       |
| update_schema_status_via_governance                      | `schema_id: SchemaId`<br/>`status: SchemaStatus`                                                                                                                    | Update a Schema's status via Governance                                |

### 12. **Runtime Calls**<a id="runtime_calls"></a>

The following new Custom Runtime functions are proposed:

| Custom Runtime Function      | Parameters                                                                                                                    | Description                                                                                                                                                    |
|------------------------------|-------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------|
| resolve_intent_or_group_name | `protocol_name: ProtocolName`<br/>`descriptor_name: Option<DescriptorName>`                                                   | Resolve a name to a registered  ID or list of IDs                                                                                                              |
| check_intent_group           | `group_id: IntentGroupId`<br/>`msa_id: MessageSourceId`<br/>`provider_id: ProviderId`<br/>`block_number: Option<BlockNumber>` | Returns the Intents currently-defined IntentGroup, mapped to a boolean indicating the current delegation status of that Intent for the given MSA and Provider. |
| get_schemas_for_intent       | `intent_id: IntendId`                                                                                                         | Return the list of Schemas that implement the indicated Intent                                                                                                 |

### 13. **Storage**<a id="storage"></a>

#### Schemas

The storage structures for Schemas will not fundamentally change, except for internal changes to the `SchemaInfo`
structure that will require migration:

* `payload_location`  & `settings` will migrate to `IntentInfo`
* addition of `intent_id`, `status`

#### Intents

The <a href="#intent_struct">IntentInfo storage</a> will be as follows:

```rust
#[pallet::storage]
pub(super) type IntentInfos<T: Config> =
StorageMap<_, Twox64Concat, IntentId, IntentInfo, OptionQuery>;
```

#### Delegations

There will be no change to Delegation storage; existing delegated `SchemaIds` will be interpreted as `IntentIds`.

#### Intent Groups

The <a href="#intent_group_struct">IntentGroup structures</a> will be stored as follows:

```rust
#[pallet::storage]
pub(super) type IntentGroups<T: Config> =
StorageMap<_, Twox64Concat, IntentGroupId, IntentGroup, OptionQuery>;
```

#### Name Registry

The <a href="#name_registry_struct">Name Registry structures</a> will look as follows:

```rust
#[pallet::storage]
pub(super) type NameRegistry<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    ProtocolName,
    Blake2_128Concat,
    NameDescriptor,
    RegisteredNameIdType,
    ValueQuery,
>;
```

#### `messages` pallet

The `messages` pallet will require new `messages::MessagesV3` storage that includes the `schema_id` used to format the
message. The new storage will still be indexed by `(BlockNumber, u16, MessageIndex)` as before. The `u16` part of the
index, however, will now represent `IntentId` rather than `SchemaId`. The update `Message` structure itself will now
contain `schema_id`.

```rust
pub struct Message<MaxDataSize>
where
    MaxDataSize: Get<u32> + Debug,
{
    ///  Data structured by the associated schema's model.
    pub payload: BoundedVec<u8, MaxDataSize>,
    /// Message source account id of the Provider. This may be the same id as contained in `msa_id`,
    /// indicating that the original source MSA is acting as its own provider. An id differing from that
    /// of `msa_id` indicates that `provider_msa_id` was delegated by `msa_id` to send this message on
    /// its behalf.
    pub provider_msa_id: MessageSourceId,
    ///  Message source account id (the original source).
    pub msa_id: Option<MessageSourceId>,
    ///  The SchemaId of the schema that defines the format of the payload
    pub schema_id: SchemaId,
}

#[pallet::storage]
pub type MessagesV3<T: Config> = StorageNMap<
    _,
    (
        storage::Key<Twox64Concat, BlockNumberFor<T>>,
        storage::Key<Twox64Concat, SchemaId>,
        storage::Key<Twox64Concat, IntentId>,
        storage::Key<Twox64Concat, MessageIndex>,
    ),
    Message<T::MessagesMaxPayloadSizeBytes>,
    ValueQuery>;
```

#### `stateful-storage` pallet

Since the actual user payloads stored in `stateful-storage` are user-defined, there will be no modification to actual
payload data. However, since storage will now be keyed by `IntentId` rather than `SchemaId`, we need to store the
associated `schema_id` with each page or item so that readers can know how to interpret the associated payload data.
This information will be added to the header of each Page and Item.

Additionally, the Page and Item header structs will contain version variant information so that future pallet evolution
need not necessitate a migration.

### Migrations

Anticipated migrations are as follows:

1. Create a new `Intent` for every currently existing `Schema`, as follows:
    1. The new `Intent` will have the same numeric ID value as the original Schema.
    2. The new Intent shall inherit the `payload_location` and `settings` from the existing `SchemaInfo` object.
2. Migrate existing schema name mappings as follows:
    1. For each `SchemaNamespace` '\<protocol_name>'
        1. For each `SchemaDescriptor` '\<name>' at index `n` belonging to a '\<protocol_name>'
            * Create a new name mapping in the `NameRegistry` as `<protocol_name>.<name>_n` to `Intent(id)`
3. Migrate `messages::MessagesV2` data to the new `messages::MessagesV3` and kill `messages::MessagesV2`
4. Migrate the _values_ stored in all `stateful-storage` pages as follows:
    * All pages (including both Paginated and Itemized pages) shall be re-written with a page header that includes, in
      addition to the payload size, an initial page version enum and `schema_id`, which will provide for future
      adaptability without forcing migrations.
    * All individual items in an Itemized page shall be re-written with a new `ItemHeader` that includes versioning
      meta-information and `schema_id`.
    * No storage keys will be transformed as part of this migration; only the values stored under each key.

#### Additional notes

A prior iteration of this document described an approach that required no migrations for either the `messages` or the
`stateful-storage` pallets. This decision was reversed for the following reasons:

* Keeping code to read many versions of the same semantic data is less maintainable.
* A `messages` pallet migration, as currently designed, incurs no disruption at all to the operaion of the pallet.
* At the time of development, given chain usage patterns for `stateful-storage`, a data migration is the least
  disruptive that it is likely to be for the forseeable future.

### Miscellaneous

The following additional changes will be made to other pallets that currently reference Schemas:

#### `msa` pallet

* All storage, extrinsics, and RPC calls that currently reference `SchemaId` shall be updated to instead reference
  `IntentId`. This will not change the shape of the public API or storage, as both types are `u16` (so SCALE encoding
  will not change). The only change will be to the runtime interpretation of arguments and return values.

#### `messages` and `stateful-storage` pallets

* All extrinsics for `writing` data shall continue to accept `SchemaId` as a parameter, as that is still required
  information for writing data, and the `IntentId` can be derived from the `Schema` at no additional runtime cost.
* All Runtime API and RPC calls shall accept `IntentId` for record retrieval, as that is what identifies the storage
  location.