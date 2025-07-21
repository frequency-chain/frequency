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
- Delegations apply to the Intent, not a specific schema version ID
    - _exception: do we continue to support delegation by `SchemaId` for backwards-compatibility?_
- _Question: can versions be deprecated?_

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
individual `SchemaId`s, but we propose expanding the delegation model to include one or more of the following:

- Delegation by `SchemaId` (existing/legacy, for future deprecation)
- Delegation by `IntentId` (implies all schema versions within the Intent)
    - Option to consider: delegation by `IntentId` + specific version/range of versions

All delegation modes above may coexist, allowing for graceful migration and flexible usage.

## 8. **Authorization Resolution Model** <a id="section_8"></a>

This section outlines how delegation is verified at the time of an extrinsic call or message submission. In the current
model, permissions are evaluated by a specific `SchemaId`. In the new model, the runtime must also support evaluating
permissions based on an `IntentId`. This delegation model will entail either:

- 1-2 additional storage reads to determine the Intent associated with a given `SchemaId`, or
- New extrinsics that require `IntentId` to be supplied in addition to `SchemaId`

### 🧪 Additional Considerations

- SDKs should provide helpers for intent resolution and delegation explanation.

## 9. **On-Chain Data Structures** <a id="section_9"></a>

This section outlines the proposed on-chain data structures to support protocols, schemas, intents, and delegation
ownership in the redesigned Frequency architecture.

### 🧱 Design Principles

- On-chain runtime operations should only work with compact numeric identifiers (`SchemaId`, `IntentId`, `ProtocolId`)
- Human-readable names must be resolved by the client prior to calling extrinsics
- Delegations are unified under a single structure, regardless of type
- Minimize storage duplication and unnecessary indices

----------

### 🧩 Delegation Structures

```rust
// NOTE: DelegationTarget is not needed if we eliminate Schema-based delegation via a storage migration
pub enum DelegationTarget {
    Schema(SchemaId),
    Intent(IntentId),
}

pub struct DelegationInfo {
    pub granted_at: BlockNumber,
    pub revoked_at: Option<BlockNumber>,
}

pub struct Delegation {
    pub revoked_at: BlockNumber,
    // NOTE: If we eliminate DelegationTarget for IntentId, we can easily restore the current implmentation's
    // BoundedBTreeMap implementation for additional runtime efficiency
    pub delegated_targets: BoundedBTreeMap<DelegationTarget, DelegationInfo, MaxTargetsPerDelegation>,
}

pub type Delegations<T> = StorageDoubleMap<
    _, Blake2_128Concat, MsaId,       // Delegator
    Blake2_128Concat, ProviderId,     // Provider
    Delegation,
>;
```

This structure handles all types of delegations (schema, intent) uniformly. Legacy `SchemaId` delegation
can optionally remain supported, with intent-based delegation layered on top.

This unified structure supports flexibility while maintaining clear access logic.<br/>
**NOTE:** this storage format approximately doubles the storage requirement for delegations (from a max of ~180 bytes
per
MSA/Provider currently, to a max of ~360 bytes for the same number of delegations). However, this size differential is
not due to the architectural difference in the storage structure, but rather the fact that we've chosen to store
additional information (`granted_at` block number) that we previously did not retain, and has been noted as a deficiency
of the current Delegation storage model. If we choose _not_ to correct that deficiency, the storage impact of this
architectural change is minimal (~30-byte increase per MSA/Provider for the maximum number of delegations)

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

### 🧹 Design Simplification

The current runtime includes a top-level flag indicating whether any delegation exists between a user and provider. This
was used to shortcut delegation checks. In this model, that summary flag is dropped — its only purpose was to optimize
for the empty case. If need be, the design can be updated to preserve this structure.

This simplification removes complexity and keeps delegation logic self-contained.

## 10. **API Identifier Handling and Resolution** <a id="section_10"></a>

The choice of identifiers in runtime and SDK APIs has significant implications for both usability and performance.
Historically, Frequency APIs used `SchemaVersionId` for delegation and data interpretation. The proposed model must
account for schemas and intents, while maintaining runtime efficiency.

### 🔢 Preferred On-Chain Identifiers

On-chain APIs and runtime logic should rely exclusively on numeric identifiers, ie `ProtocolId` and `IntentId` rather
than `protocol_name.intent_name`.

String identifiers such as protocol names or intent names should never be parsed on-chain. These must
be resolved by SDKs and off-chain services (e.g., the Frequency Gateway).

### 🧠 Resolution Responsibility

Clients and SDKs must perform resolution of strings to IDs before invoking extrinsics. For example:

- Resolve `"dsnp.broadcast"` → `(ProtocolId, IntentId)`

### 📬 Extrinsic Parameters

Three possible design patterns:

1. **Submit by SchemaId only (Legacy/Compatible):**

    - Fast, direct lookup
    - Requires on-chain schema metadata to determine intent for validation
    - May require multi-delegation validation (intent or schema)

2. **Submit by SchemaId + IntentId (Explicit):**

    - Enables precision delegation checks
    - Slightly more complex client integration

3. **Submit by IntentId only (No Schema):**

    - Enables non-schema-based actions (e.g., future custom actions)
    - Runtime must validate delegation by `IntentId`

**NOTE:** Minimally, `SchemaId` _must_ be provided for all use cases that involve writing data to be associated with a
schema, because the specific `SchemaId` must be recorded in the data or event payload somehow. There are no current use
cases for extrinsics that expect _only_ an `IntentId`.

### 🎯 Developer Experience Considerations

- APIs should remain backward compatible with `SchemaId`-based extrinsic calls.
- Whether to expose intent-based extrinsics directly is an open question.
- SDKs such as the Frequency Gateway may offer string-based input and resolve IDs internally.

## 11. **Migration Strategy** <a id="section_11"></a>

Transitioning from the current schema delegation system to the new schema + intent model must be handled
carefully to preserve backward compatibility while enabling the new architecture to roll out incrementally.

### 🧩 Legacy Support

- Legacy `SchemaId`-based delegations will continue to be supported for existing providers
- Existing data using older schema versions remains valid
- Providers can continue operating without changes while the new system is phased in

### 🚀 Protocol Bootstrapping

- Governance/migration may backfill `IntentId`s and `ProtocolId`s for existing DSNP schemas
    - Each legacy schema version may be added to its corresponding schema version array
    - Historical schema versions will be placed in schema version order

### 🔁 Delegation Migration

- Providers can request users re-authorize using `IntentId` delegations at their discretion
- SDK tooling may provide utilities to convert legacy schema delegations into intent equivalents

### 🛑 Sunset of Schema-Based Delegation (Future)

Once adoption of intent/protocol delegation is widespread:

- Chain governance may disable new schema-based delegations
- Existing schema-based delegations may be invalidated at a future block height
- Tooling will issue deprecation warnings ahead of this change

This strategy ensures a smooth, opt-in path to the new system while preserving all critical functionality during the
transition period.

## 12. **Open Questions and Next Steps** <a id="section_12"></a>

Several open questions remain that may influence the final implementation strategy and runtime behavior. These are left
intentionally unresolved to guide future design discussions within the development and governance communities.

### ❓ Open Questions

- Is it possible to devise a migration strategy that is backwards-compatible but somehow migrates current delegations to
  Intent-based, thereby allowing us to immediately deprecate SchemaId-based delegation?

- What developer-facing tools will be needed to ease migration and debugging?

- Will schema deprecation or version invalidation ever be supported, or are all published schemas permanently
  active?

- This design does not address the following shortcoming in the delegation model:
  If a user delegates to a Provider, then revokes that delegation, and subsequently adds the same delegation again, we
  lose information about the first period delegation. Example:
    - User A delegates Intent 1234 to Provider 'Alice' at block 100
    - User A revokes delegation at block 200.
        - We are able to determine that content posted by Alice on behalf of User A between blocks 100-200 is valid;
          conversely, we know that any content posted outside the block range of 100-200 was not authorized
    - User A re-delegates Provider Alice at block 400
        - We are able to determine the validity of content posted by Alice on behalf of User A for blocks >= 400.
          However, we have now lost the information about the prior validity period of blocks 100-200, and can no longer
          recognize that content posted in that block range was authorized. (Presumably, we could still make that
          determination if we have access to an archive node and query the storage at the block number at which the
          content was posted...)

### 🧭 Next Steps

1. **Internal review** of this proposal by Frequency core contributors

2. **Discussion of delegation scope models** and confirmation of preferred resolution pattern

3. **Engineering spike** into schema/intents/protocol runtime storage cost and migration

4. **API/SDK prototype** demonstrating resolution and delegation lookup UX

5. **Drafting of runtime implementation plan** and migration support utilities

6. **Call for feedback** from developers, partners, and governance stakeholders

This document serves as a foundation for these efforts and should evolve alongside prototyping and implementation
planning.
