# 📘 Design Discussion: Schema, Protocols, and Intent-Based Delegation in Frequency

## 0. **Work in Progress** <a id="section_0"></a>

Note: This document is a work in progress; specific implementation details and code examples exhibited herein are for
illustrative purposes only. Once the various questions and concerns surfaced by this "pre-design" document have be
answered satisfactorily, the document will be updated and expanded to include specific details related to the proposed
implementation.

## 1. **Background and Motivation** <a id="section_1"></a>

In the current implementation, schemas are registered with immutable numeric identifiers (`SchemaId`) and describe the
layout and storage semantics (e.g., Avro/Parquet formats, on-chain/off-chain storage). These schema IDs are used as
references by clients and runtime modules alike, particularly in the delegation system defined by the `msa` pallet.

Delegations currently allow a user to authorize a provider (e.g., an app or service) to act on their behalf, but this
authorization is tightly bound to a specific `SchemaId`. This model has proven limiting in several ways:

- **Coupling between schema versions and delegations**
- **Schemas represent data format, not purpose**
- **Lack of human-readable context**

These limitations have motivated a re-architecture of the schema and delegation systems to introduce the concepts of:

- **Named intents** with version tracking
- **Intent-based delegation**
- **More expressive APIs and storage models**

## 2. **Design Goals** <a id="section_2"></a>

This section outlines the key objectives that guide the redesign of Frequency's schema and delegation architecture.

- **Schema Immutability** - Individual schema versions, once published, are immutable on-chain
- **Minimal Delegation Churn** - Minor changes to data formats should not require new delegations
- **Minimal Storage Churn (migrations)** - Minor changes to storage formats should not require mass migration of user
  data
- **Intent Separation** - When permissioning, we need to be able to separate the purpose of the data and action from its
  format
- **On-Chain Efficiency** - On-chain operations need to be efficient, so storage and structures must be designed with
  that in mind

## 3. Glossary of Terms <a id="section_3"></a>

- **Protocol** - the top level of the tripartite nomenclature _protocol.intent@version_ (ex: _dsnp_ is the protocol in
  _dsnp.broadcast@v2_)
- **Intent** - the second level of _protocol.intent@version_ (ex: _dsnp.broadcast_ resolves to a specific `Intent`).
  Intents should always be referred to by their fully-qualified name (ie, _dsnp.broadcast_, not just _broadcast_).
  Intents represent some meaning or purpose that may be delegated, and may optionally be associated with one or more
  _minor_ versions of a `Schema`
- **version** \[optional] - the third level of _protocol.intent@version_, resolves to a specific `SchemaId`
- **Schema** - A `Schema` represents a particular data format, along with additional metadata about its location, etc

## 4. **Protocols** <a id="section_4"></a>

In this design, a `Protocol` is merely an organizational tool for collections of Intents. In a future design update, a
Protocol may be promoted to a first-level on-chain entity with a numeric ID, or, it may be eliminated completely in
favor of an ENS or ENS-like system on Frequency.

<a id="protocol_map"></a>Protocols will be represented on-chain in a manner consistent with the current implementation
of schema names; _ie_, a double storage map `protocol name` -> `intent name` -> `IntentId`

## 5. **Intents** <a id="section_5"></a>

Intents are not just organizational tools—they enable meaningful delegation. In the new model, an Intent is a
first-class on-chain entity. It can represent a single action or permission that may be delegated; it can optionally
reference a list of Schemas that comprise minor version updates of a particular data format--in the latter case, the
implied permission is `schema.Write`. Every published Schema is assigned to an Intent, and new versions of a schema
simply add new schema IDs to the Intent's version history.

As mentioned above, an `Intent` may reference a list of `Schemas` that comprise the sequential _versions_ to which a
delegation to the Intent grants _write_ access. However, it is also possible to create an Intent that does not reference
_any_ Schemas; such an Intent may be used as a way to record arbitrary permissions on-chain. These permission indicators
may be used by off-chain applications (or future on-chain facilities). For example, a wallet application may query an
on-chain Intent to determine whether an application should be allowed access to a user's graph encryption key.

### 📌 Intent Metadata Structure

- `protocol_name**`: Which Protocol owns this Intent
- `name**`: Human-readable name (e.g., `broadcast`)
- `description`: Optional developer-facing documentation
- `link`: Optional link to spec or docs
- `registered_by`: MSA (ProviderId) that registered the intent
- `registered_at`: Block number
- `schema_versions`: [optional] Ordered list of `SchemaId`s (e.g., `[7, 15, 27]`, for versions 0..2)

_\*\*may be removed from metadata struct as it would be redundant due to <a href="#protocol_map">Protocol storage
map</a>_

### 🧬 Versioning Rules

- Versions are stored in a monotonic, gapless array
- A new SchemaId is automatically assigned to the next version in the Intent
- Older versions are preserved permanently
- Delegations apply to an Intent
- _Question: can Schemas (particular versions) be deprecated?_

### 🧭 Governance Rules

- Intents must be approved by governance, and are immutable except for appending new SchemaIds to the version list
- New Schemas must be approved by governance
- Minor Schema updates (semantic-preserving format changes, etc) may be approved for the same Intent
- Major updates (change in meaning or semantics, or significant breaking format change) require publishing under a new
  Intent

## 6. **Schemas** <a id="section_6"></a>

`Schemas` are on-chain entities that describe a data format and storage location. Under this new design, Schemas are
little changed from their current implementation, conceptually, although some on-chain storage may be mutated for
efficiency, as well as the addition of some metadata.

### 🧭 Governance Rules

- Schemas, as currently, must be approved by governance
- A published Schema must reference an existing Intent

## 7. **Delegation Semantics** <a id="section_7"></a>

Delegation is the mechanism by which a user authorizes a provider to act on their behalf. Currently, this is limited to
individual `SchemaId`s, but we propose changing the delegation model to the following:

- Delegation by `IntentId` (implies all schema versions within the Intent)
    - Option to consider: delegation by `IntentId` + specific version/range of versions

## 8. **Authorization Resolution Model** <a id="section_8"></a>

This section outlines how delegation is verified at the time of an extrinsic call or message submission. In the current
model, permissions are evaluated by a specific `SchemaId`. In the new model, permissions are based on an `IntentId`.

### 🧪 Additional Considerations

- SDKs should provide helpers for intent resolution and delegation explanation.

## 9. **On-Chain Data Structures** <a id="section_9"></a>

This section outlines the proposed on-chain data structures to support protocols, schemas, intents, and delegation
ownership in the redesigned Frequency architecture.

### 🧱 Design Principles

- On-chain runtime operations should only work with compact numeric identifiers (`SchemaId`, `IntentId`, `ProtocolId`)
- Human-readable names must be resolved by the client prior to calling extrinsics
- Minimize storage duplication and unnecessary indices

----------

### 🧩 Delegation Structures

The on-chain storage structures for Delegations do not change as a result of this design, other than the fact that the
`SchemaId` reference currently stored becomes an `IntentId` reference of the same value.
----------

### 🏷 Intents and Protocols

```rust
pub type IntentNameToIds<T: Config> = StorageDoubleMap<
    _,
    Blake2_128Concat,
    ProtocolName,
    Blake2_128Concat,
    IntentName,
    IntentId,
    ValueQuery,
>;

pub struct IntentInfo {
    description: BoundedVec<u8, ConstU32<DESCRIPTION_MAX>>,
    link: BoundedVec<u8, ConstU32<URL_MAX>>,
    registered_at: BlockNumber,
    schema_versions: BoundedVec<SchemaId, MaxSchemaVersionsPerIntent>,
}

pub type Intents<T> = StorageMap<
    _, Blake2_128Concat, IntentId, IntentInfo
>;
```

Note: `IntentIds` are only referenced by numeric ID. Clients must resolve names via off-chain lookup using mappings
from `protocol name + intent name → IntentId`.

----------

## 10. **API Identifier Handling and Resolution** <a id="section_10"></a>

The choice of identifiers in runtime and SDK APIs has significant implications for both usability and performance.
Historically, Frequency APIs used `SchemaId` both for delegation and data interpretation. The proposed model must
account for schemas and intents, while maintaining runtime efficiency.

### 🔢 Preferred On-Chain Identifiers

On-chain APIs and runtime logic should rely exclusively on numeric identifiers, ie `IntentId`, and `SchemaId` rather
than `protocol_name.intent_name`.

String identifiers such as protocol names or intent names should never be parsed on-chain. These must
be resolved by SDKs and off-chain services (e.g., the Frequency Gateway).

### 🧠 Resolution Responsibility

Clients and SDKs must perform resolution of strings to IDs before invoking extrinsics. For example:

- Resolve `"dsnp.broadcast"` → `IntentId`

### 📬 Extrinsic Parameters

Supported design patterns:

1. **Submit by SchemaId only (Legacy/Compatible):**
    - Requires on-chain schema metadata to determine intent for validation
        - this is already read in the current flow, so no additional reads
    - Given the `IntentId` retrieved from the `SchemaInfo`, delegation lookup proceeds as currently

2. **Submit by IntentId only (No Schema/future extrinsics):**
    - Enables non-schema-based actions (e.g., future use cases)

**NOTE:** Minimally, `SchemaId` _must_ be provided for all use cases that involve writing data to be associated with a
schema, because the specific `SchemaId` must be recorded in the data or event payload somehow. There are no current use
cases for extrinsics that expect _only_ an `IntentId`.

## 11. **On- and Off-chain Schema Payloads & Storage** <a id="section_11"></a>

The change from `Schema` to `Intent` has implications how and where data is stored, especially as currently relating to
the `messages` and `stateful-storage` pallets.

The `messages` pallet stores both on- and off-chain messages in the following structure:

```rust
    #[pallet::storage]
pub(super) type MessagesV2<T: Config> = StorageNMap<
    _,
    (
        storage::Key<Twox64Concat, BlockNumberFor<T>>,
        storage::Key<Twox64Concat, SchemaId>,
        storage::Key<Twox64Concat, MessageIndex>,
    ),
    Message<T::MessagesMaxPayloadSizeBytes>,
    OptionQuery,
>;
```

The `stateful-storage` pallet stores data in a child trie keyed on `SchemaId`.

Both pallets' storage is physically keyed at least partially on `SchemaId`. This makes it easy, for instance, to query
all of the messages for a particular schema. However, one of the issues this design is meant to resolve is the necessity
for a storage migration any time there is a minor schema version published.

The `messages` pallet actually does not currently require a migration in this scenario; existing messages are accessible
via the old schema version, while new messages are written via the new minor version. However, this does make querying
for content somewhat cumbersome. In a scenario where there have been 5 minor schema version updates to the
`'dsnp.broadcast'` intent, the current model would necessitate 5 separate queries to the chain to ensure we retrieved
all
valid `'dsnp.broadcast'` content.

The `stateful-storage` pallet presents a different challenge. `PaginatedStorage`, for instance, is intended to be able
to serialize a single, consistent data model (split across multiple pages). Currently, even a minor schema format change
requires the entire model to be migrated to a new child trie under the new `SchemaId`; otherwise, data inconsistencies
can arise, as well as other problems.

To mitigate these issues, we will store data indexed by `IntentId` instead of `SchemaId`. This will require the
encapsulated storage payload to contain an additional piece of meta-information; specifically, the concrete `SchemaId`
that was used to format the encapsulated data.

### 🎯 Developer Experience Considerations

- APIs should remain backward compatible with `SchemaId`-based extrinsic calls.
- SDKs such as the Frequency Gateway may offer string-based input and resolve IDs internally.

## 12. **Migration Strategy** <a id="section_12"></a>

Transitioning from the current schema delegation system to the new intent delegation model must be handled
carefully to preserve backward compatibility while enabling the new architecture to roll out incrementally.

### 🚀 Protocol and Intent Bootstrapping

For each Schema currently registered on-chain:

- Create a new Intent with the same numeric `IntentId` value as the current `SchemaId`
- If the `SchemaId` is referenced in the existing `SchemaNamesToIds` mapping
    - If the `SchemaId` _is not_ the latest registered version in the mapping array, register the new Intent with the
      same   `protocol.intent` name, appending `'_v{version_index}'` to the name
    - If the `SchemaId` _is_ the latest registered version in the mapping array, register the new Intent with the same
      `protocol.intent` name (no suffix)
- If the `SchemaId` is not registered with a name mapping
    - Create a new mapping `'global.schema_{id}'`
    - Alternately, since this scenario should only exist on Testnet, drop the schema
- Rewrite the Schema's metadata on-chain to reference the corresponding `IntentId` (seems redundant, as existing
  `SchemaId` == `IntentId`, but future Schemas will not preserve that relationship)

This migration approach should ensure that existing Delegation references do not need to be migrated, as all new Intents
will have the same numeric IDs as the existing Schemas.

### 🚀 Data Payload Migration

Migration of both `messages` and `stateful-storage` pallet data is necessary, due to the new requirement to store the
concrete `SchemaId` with the payload.

For the `messages` pallet, it _may_ be possible to avoid a migration by bifurcating message storage at a specific block
number, ie, content in blocks before the upgrade are kept in the existing storage keyed by `SchemaId`, while new content
is stored indexed by `IntentId` and containing a `SchemaId` in the payload. Further analysis is required to determine if
this approach is feasible or desirable.

For the `stateful-storage` pallet, it would be necessary to re-write all storage pages to include the concrete
`SchemaId` in the page header. How to accomplish this for ~1M user graphs and graph keys on-chain requires further
analysis.

## 12. **Open Questions and Next Steps** <a id="section_12"></a>

Several open questions remain that may influence the final implementation strategy and runtime behavior. These are left
intentionally unresolved to guide future design discussions within the development and governance communities.

### ❓ Open Questions

- What developer-facing tools will be needed to ease migration and debugging?

- Will schema deprecation or version invalidation ever be supported, or are all published schemas permanently
  active?

- This design specifically does not address the issue described
  in [#2510](https://github.com/frequency-chain/frequency/issues/2510)

### 🧭 Next Steps

1. **Internal review** of this proposal by Frequency core contributors

2. **Discussion of delegation scope models** and confirmation of preferred resolution pattern

3. **Engineering spike** into schema/intents/protocol runtime storage cost and migration

4. **API/SDK prototype** demonstrating resolution and delegation lookup UX

5. **Drafting of runtime implementation plan** and migration support utilities

6. **Call for feedback** from developers, partners, and governance stakeholders

This document serves as a foundation for these efforts and should evolve alongside prototyping and implementation
planning.
