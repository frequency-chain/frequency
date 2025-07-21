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
- **Rigid governance workflows**
- **No concept of publisher authority**

These limitations have motivated a re-architecture of the schema and delegation systems to introduce the concepts of:

- **Named schemas** with version tracking
- **Structured publishing under protocols**
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
- **Scoped Delegations** - Delegations should be able to target either a specific schema version or a complete
  intent (collection of schemas)
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

In order to provide organizational control for schemas and intents, we introduce the concept of **Protocols**.

A Protocol (e.g., `dsnp`, `bsky`) serves as a root authority under which Schemas and Intents are registered.
Protocols allow for Intents and Schemas with similar meaning or structure, but different purposes, to be delegated
in a scoped manner. For example, there could be both a 'dsnp.broadcast' and a 'bsky.broadcast' Intent; both have similar
meaning ('broadcast a message'), but the Protocol differentiates between broadcasting a DSNP message vs. an AT
Protocol (ie, Bluesky) message.

### 📛 Protocol Structure

Each protocol is a unique identifier, typically human-readable (e.g., `"dsnp"`), mapped to an on-chain `ProtocolId` (
e.g., `ProtocolId = 2`).

- Every Intent must be published under a protocol: `protocol.intent`

### ⚙️ Protocol Governance

- **Creation of a new protocol** must be approved via governance.
- Protocols are immutable once registered

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

- `IntentId`: A unique numeric identifier for runtime efficiency
- `ProtocolId`: Which Protocol owns this Intent
- `Name`: Human-readable name (e.g., `broadcast`)
- `Description`: Optional developer-facing documentation
- `Link`: Optional link to spec or docs
- `RegisteredBy`: MSA (ProviderId) that registered the intent
- `RegisteredAt`: Block number
- `Versions`: [optional] Ordered list of `SchemaId`s (e.g., `[7, 15, 27]`, for versions 0..2)

### 🧬 Versioning Rules

- Versions are stored in a monotonic, gapless array
- A new SchemaId is automatically assigned the next version in the Intent
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
pub type ProtocolInfos<T> = StorageMap<
    _, Blake2_128Concat, ProtocolId, ProtocolInfo
>;

pub struct IntentInfo {
    protocol_id: ProtocolId,
    name: BoundedVec<u8, ConstU32<IDENTIFIER_MAX>>,
    description: BoundedVec<u8, ConstU32<DESCRIPTION_MAX>>,
    link: BoundedVec<u8, ConstU32<URL_MAX>>,
    registered_at: BlockNumber,
}

pub type Intents<T> = StorageMap<
    _, Blake2_128Concat, IntentId, IntentInfo
>;

// Q: Is this necessary as a separate map, or can be folded into the `IntentInfo` struct?
pub type IntentVersions<T> = StorageMap<
    _, Blake2_128Concat, IntentId, BoundedVec<SchemaId, MaxSchemaVersionsPerIntent>
>;
```

Note: `IntentIds` are only referenced by numeric ID. Clients must resolve names via off-chain lookup using mappings
from `ProtocolId/protocol name + intent name → IntentId`.

----------

## 10. **API Identifier Handling and Resolution** <a id="section_10"></a>

The choice of identifiers in runtime and SDK APIs has significant implications for both usability and performance.
Historically, Frequency APIs used `SchemaId` both for delegation and data interpretation. The proposed model must
account for schemas and intents, while maintaining runtime efficiency.

### 🔢 Preferred On-Chain Identifiers

On-chain APIs and runtime logic should rely exclusively on numeric identifiers, ie `ProtocolId`, `IntentId`, and
`SchemaId` rather than `protocol_name.intent_name`.

String identifiers such as protocol names or intent names should never be parsed on-chain. These must
be resolved by SDKs and off-chain services (e.g., the Frequency Gateway).

### 🧠 Resolution Responsibility

Clients and SDKs must perform resolution of strings to IDs before invoking extrinsics. For example:

- Resolve `"dsnp.broadcast"` → `(ProtocolId, IntentId)`

### 📬 Extrinsic Parameters

Three possible design patterns:

1. **Submit by SchemaId only (Legacy/Compatible):**

    - Requires on-chain schema metadata to determine intent for validation
        - this is already read in the current flow, so no additional reads
    - Given the `IntentId` retrieved from the `SchemaInfo`, delegation lookup proceeds as currently

3. **Submit by IntentId only (No Schema/future extrinsics):**

    - Enables non-schema-based actions (e.g., future use cases)

**NOTE:** Minimally, `SchemaId` _must_ be provided for all use cases that involve writing data to be associated with a
schema, because the specific `SchemaId` must be recorded in the data or event payload somehow. There are no current use
cases for extrinsics that expect _only_ an `IntentId`.

### 🎯 Developer Experience Considerations

- APIs should remain backward compatible with `SchemaId`-based extrinsic calls.
- SDKs such as the Frequency Gateway may offer string-based input and resolve IDs internally.

## 11. **Migration Strategy** <a id="section_11"></a>

Transitioning from the current schema delegation system to the new intent delegation model must be handled
carefully to preserve backward compatibility while enabling the new architecture to roll out incrementally.

### 🚀 Protocol and Intent Bootstrapping

- Migration may backfill `IntentId`s and `ProtocolId`s for existing DSNP schemas
    - For each existing `SchemaId`, a corresponding `Intent` will be created with the same numeric ID value, so that
      existing Delegation references do not need to be migrated

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
