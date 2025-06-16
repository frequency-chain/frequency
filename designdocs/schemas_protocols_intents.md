# 📘 Design Discussion: Schema Protocols and Intent-Based Delegation in Frequency

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

- **Schema Immutability**
- **Protocol-Based Versioning**
- **Minimal Delegation Churn**
- **Intent Separation**
- **Scoped Delegations**
- **Protocol Ownership and Governance**
- **On-Chain Efficiency**

## 3. Glossary of Terms <a id="section_3"></a>

- **protocol** - the top level of the tripartite nomenclature _protocol.schema@version_ (ex: _dsnp_ is the protocol in
  _dsnp.broadcast@v2_)
- **schema** - the second level of _protocol.schema@version_ (ex: _dsnp.broadcast_ resolves to a specific schema).
  Schemas should always be referred to by their fully-qualfied name (ie, _dsnp.broadcast_, not just _broadcast_)
- **version** - the third level of _protocol.schema@version_, resolves to a specific _minor_ iteration of a schema
- **intent** - an on-chain primitive representing an abstract action or operation

## 4. **Schemas and Versioning** <a id="section_4"></a>

Schemas are not just organizational tools—they enable meaningful delegation. In the new model, a schema is a
first-class on-chain entity that represents a named and versioned group of _schema versions_. Every schema version is
assigned to a
schema, and new versions of a schema simply add new schema version IDs to the schema's version history.

### 📌 Schema Metadata Structure

- `SchemaId`: A unique numeric identifier for runtime efficiency
- `ProtocolId`: Which protocol owns this schema
- `Name`: Human-readable schema name (e.g., `broadcast`)
- `Versions`: Ordered list of `SchemaVersionId`s (e.g., v0 → v1 → v2)
- Metadata: Optional description, links, etc.

### 🧬 Versioning Rules

- Versions are stored in a monotonic, gapless array
- A new schema version ID is automatically assigned the next version
- Older versions are preserved permanently
- Delegations may apply to the schema, not a specific schema version ID
- _Question: can versions be deprecated?_

### 🧭 Governance Rules

- New schemas must be approved by governance
- Minor updates (new schema version) may be published by the protocol owner
- Major updates require a new schema (e.g., change in semantic intent)

## 5. **Protocol Ownership** <a id="section_5"></a>

In order to provide organizational control, accountability, and publishing boundaries for schemas and intents, the
protocol introduces the concept of **protocols**.

A protocol (e.g., `dsnp`, `bsky`) serves as a root authority under which schemas and intents are registered.
Protocols allow developers and governance bodies to clearly define who may publish what, and to prevent protocol
squatting or unauthorized use of shared protocol identifiers.

### 📛 Protocol Structure

Each protocol is a unique identifier, typically human-readable (e.g., `"dsnp"`), mapped to an on-chain `ProtocolId` (
e.g., `ProtocolId = 2`).

- Every schema must be published under a protocol: `protocol.schema`
- Intents may also be published under a protocol _(open question, discussed [below](#section_7))_

### 👤 Ownership and Control

Protocols are envisioned to be owned by an entity, but there is an open question as to whether this entity should be an
MSA (i.e., a Provider) or a raw account (i.e., a public key). This decision will impact how access is managed,
especially in governance and delegation flows.

If protocols are owned by MSAs, they inherit support for delegation and signature validation via MSA structures. If
they are owned by raw accounts, they align more directly with typical on-chain identity models and may simplify some
access control paths.

Regardless of the final model, the owner of a protocol has exclusive authority to:

- Publish schemas and schema versions under the protocol
- Register intents (namespaced or global) scoped to the protocol
- Transfer protocol ownership (via governance or extrinsic)
- Assign trusted publishers or revoke their rights _(future enhancement)_

### ⚙️ Protocol Governance

- **Creation of a new protocol** must be approved via governance.
- Once approved, a protocol is permanently registered and owned by an MSA.
- Future governance tracks may support delegation or stewardship models (e.g., rotating maintainers, multisig groups).

### 🔐 Protocol-Based Security Boundaries

Protocols define logical and administrative isolation between ecosystems. For example:

- `dsnp.broadcast` is owned by the DSNP team
- `bsky.profile` is controlled by Bluesky
- No party may publish schemas or intents into another's protocol namespace without explicit permission

This structure provides a trust boundary that maps to real-world organizational authority.

## 6. **Delegation Semantics** <a id="section_6"></a>

Delegation is the mechanism by which a user authorizes a provider to act on their behalf. Currently, this is limited to
individual `SchemaId`s, but we propose expanding the delegation model to include one or more of the following:

- Delegation by `SchemaVersionId` (existing/legacy, for future deprecation)
- Delegation by `SchemaId` (implies all schema versions within the schema)
- Delegation by `IntentId` (e.g., “message.publish”), with optional protocol namespace scoping

All three modes may coexist, allowing for graceful migration and flexible usage.

Each delegation will include:

```rust
pub struct DelegationInfo {
    pub entity: DelegationTarget,          // SchemaVersion, Schema, or Intent(+Protocol Namespace)
    pub revoked_at: Option<BlockNumber>,   // Replaces `expiration` and `revoked`
    pub granted_at: BlockNumber,
}
```

And:

```rust
pub enum DelegationTarget {
    SchemaVersion(SchemaVersionId),
    Schema(SchemaId),
    Intent { id: IntentId, namespace: Option<NamespaceId> },
}
```

Delegation lookup is flattened into:

```rust
pub type Delegations<T> = StorageDoubleMap<
    _, Blake2_128Concat, MsaId,           // Delegator
    Blake2_128Concat, ProviderId,         // Provider
    BoundedVec<DelegationInfo, MaxDelegationsPerProvider>
>;
```

This unified structure supports flexibility while maintaining clear access logic.

## 7. **Intent Registration & Delegation Models** <a id="section_7"></a>

Intents represent abstract actions or operations that a provider may be authorized to perform on behalf of a user. While
schemas define how data is structured, intents define what that data _means_ or how it is _used_.

For example, `message.publish` might correspond to submitting a post to a public timeline, regardless of whether it's
encoded using `broadcast`, `reply`, or `reshare` schemas.

We explore three models for intent registration and delegation:

### 🔹 Model A: Chain-Global Intents

- Intents are globally registered, usable across protocol namespaces
- A single `IntentId` (e.g., `message.publish`) applies to all schemas that implement it

**Pros**:

- Simple
- Reusable across protocols
- Easy for SDKs to reference

**Cons**:

- No isolation between ecosystems
- Potential name conflicts or governance tension

### 🔸 Model B: Protocol-Scoped Intents

- Intents are registered under a specific protocol (e.g., `dsnp.message.publish`)
- Each protocol manages its own intent catalog

**Pros**:

- Strong isolation
- Prevents cross-ecosystem collisions

**Cons**:

- Duplicative
- Harder for SDKs to unify

### 🔷 Model C: Hybrid (Recommended)

- Intents are globally registered (e.g., `message.publish`)
- Delegations may scope usage to specific protocols (e.g., `message.publish` for `dsnp` only)

**Pros**:

- Shared meaning
- Protocol-level trust boundary
- Cleanest balance of safety plus reuse

**Cons**:

- Slightly more complex resolution logic
- Requires schema metadata to track which intents are implemented

### 🚫 Considered and Dismissed: Hybrid 2 — Both Global and Protocol-Scoped Intents

- Intents may be either global or protocol-scoped
- Global intents require governance approval
- Protocol-scoped intents may be published by the protocol owner

**Issue:** This model creates potential ambiguity and collision between global and protocol-scoped intents sharing the
same
base name (e.g., both `message.publish` and `dsnp.message.publish`). It also complicates the future introduction of
global intents if a protocol-scoped version already exists.

Since the only expected use case for protocol-scoped intents is to fill the gap where no global intent exists, the
cleaner
long-term solution is to require governance-approved global-only intents, while using delegation scoping to preserve
control.

## 8. **Intent Registry Design** <a id="section_8"></a>

The intent registry is an on-chain mapping of supported intents, including associated metadata and governance control.

### 🗂 Intent Metadata

Each intent has the following attributes:

- `IntentId`: Compact numeric ID used on-chain
- `Name`: Human-readable name (e.g., `message.publish`)
- `Description`: Optional developer-facing documentation
- `Link`: Optional link to spec or docs
- `RegisteredBy`: MSA that registered the intent
- `RegisteredAt`: Block number
- _Question: can intents be deprecated?_

Intents are immutable once registered. Only governance can publish new global intents.

**Question:** should the intent registry be multi-level (ie `message` -> [`publish`, `react`])?
**Pros:**

* Makes it possible to delegate to `message.*`

**Cons:**

* Increased complexity
* Storage and runtime delegation resolution costs?

### 📥 Schema → Intents Mapping

Schemas declare the list of intents they support in their metadata. This association is stored on-chain and must be
fixed at the time of schema publication. This allows the runtime to evaluate whether a provider acting under a certain
intent has been delegated for the schema being used.

```rust
pub type SchemaIntents<T: Config> = StorageMap<
    _, Blake2_128Concat, SchemaId,
    BoundedVec<IntentId, MaxIntentsPerSchema>
>;
```

This mapping supports intent-level delegation by ensuring that intent-level delegation can be efficiently resolved from
a `SchemaId`.

### 🔍 Intent-to-Schema Lookup

For auditing, debugging, and SDK tooling, it may also be helpful to maintain the reverse map:

```rust
pub type IntentSchemas<T: Config> = StorageMap<
    _, Blake2_128Concat, IntentId,
    BoundedVec<SchemaId, MaxSchemasPerIntent>
>;
```

This map is non-critical for runtime logic but useful for ecosystem tools.

### 🔐 Governance Control

Depending on the model chosen for intents (see Section 6), intent publication would work differently:

- Global intents: Intents are only publishable by governance.
- Protocol-scoped intents: protocol owners may register intents

## 9. **Authorization Resolution Models** <a id="section_9"></a>

This section outlines how delegation is verified at the time of an extrinsic call or message submission. In the new
model, the runtime may evaluate permissions based on either a specific `SchemaVersionId`, or based on a broader context
such as
`IntentId` or `SchemaId`.

### 🔁 Delegation Type Considerations

Runtime costs are probably the deciding factor in whether to support protocol-based delegation, intents-based
delegation, or both. The resolution logic depends on the type of delegation being supported:

- **Schema-based delegation**:
    - No additional information is required.
    - Every `SchemaVersionId` maps to a single `SchemaId`, so the runtime can directly validate whether the provider is
      authorized under the protocol.

- **Intent-based delegation**:
    - If only a `SchemaVersionId` is provided, the runtime will retrieve the list of declared intents.
    - The runtime must then determine whether **any** or **all** of the schema’s intents must be delegated to the
      provider. This policy must be clearly defined and consistent across use cases.
    - If an `IntentId` is explicitly provided, the runtime can perform a targeted check.

- **SchemaVersion-based delegation**:
    - Supported for backward compatibility. Schema-level delegations override intent and schema resolution when
      present.

In practice, the resolution model may need to check **multiple types of delegation** in parallel. For example:

- If both intent-based and schema-based delegations are supported, the runtime may validate a call as authorized if *
  *either** a valid intent **or** schema delegation is found.
- The delegation resolution policy (e.g., precedence, fallback order, or required combinations) must be explicitly
  defined by the runtime and documented for developers.

This multi-path resolution allows greater flexibility, but requires care to avoid ambiguity and ensure consistent
behavior across different APIs and use cases.

### 🧪 Additional Considerations

- In the future, extrinsics may support intent-based authorization **without referencing a schema** at all. These would
  rely solely on explicit `IntentId` + optional protocol context.
- If a schema supports multiple intents, fallback rules and audit logic must be clear.
- SDKs should provide helpers for intent resolution and delegation explanation.

## 10. **On-Chain Data Structures** <a id="section_10"></a>

This section outlines the proposed on-chain data structures to support protocols, schemas, intents, and delegation
ownership in the redesigned Frequency architecture.

### 🧱 Design Principles

- On-chain runtime operations should only work with compact numeric identifiers (`SchemaVersionId`, `SchemaId`,
  `IntentId`,
  `ProtocolId`)
- Human-readable names must be resolved by the client prior to calling extrinsics
- Delegations are unified under a single structure, regardless of type
- Minimize storage duplication and unnecessary indices

----------

### 🧩 Delegation Structures

```rust
pub enum DelegationTarget {
    SchemaVersion(SchemaVersionId),
    Schema(SchemaId),
    Intent { id: IntentId, namespace: Option<ProtocolId> },
}

pub struct DelegationInfo {
    pub entity: DelegationTarget,
    pub revoked_at: Option<BlockNumber>,
    pub granted_at: BlockNumber,
}

pub type Delegations<T> = StorageDoubleMap<
    _, Blake2_128Concat, MsaId,       // Delegator
    Blake2_128Concat, ProviderId,     // Provider
    BoundedVec<DelegationInfo, MaxDelegationsPerProvider>
>;
```

This structure handles all types of delegations (schema, protocol, intent) uniformly. Legacy `SchemaVersionId`
delegation
remains supported, with protocol and intent-based delegation layered on top.

----------

### 🏷 Schemas and Protocols

```rust
pub type ProtocolInfos<T> = StorageMap<
    _, Blake2_128Concat, ProtocolId, ProtocolInfo
>;

// Question: should we have a map of ProtocolId => Owner

pub type Protocols<T> = StorageMap<
    _, Blake2_128Concat, ProtocolId,
    BoundedVec<SchemaId, MaxSchemasPerProtocol>
>;

pub type SchemaVersions<T> = StorageMap<
    _, Blake2_128Concat, SchemaId,
    BoundedVec<SchemaVersionId, MaxVersionsPerSchema>
>;

// Useful for efficient reverse-map of SchemaVersion to Schema
// needed for checking protocol delegations
pub type VersionedSchemas<T> = StorageMap<
    _, Blake2_128Concat, SchemaVersionId, SchemaId
>;
```

Note: `SchemaIds` are only referenced by numeric ID. Clients must resolve names via off-chain lookup using mappings
from `ProtocolId + name → SchemaId`.

----------

### 🧠 Intent Registry

```rust
pub type Intents<T> = StorageMap<
    _, Blake2_128Concat, IntentId, IntentMetadata
>;

// This info is contained in the SchemaInfo metadata, but
// a separate map helps with efficient lookup for intent-based delegation
pub type SchemaVersionIntents<T> = StorageMap<
    _, Blake2_128Concat, SchemaVersionId,
    BoundedVec<IntentId, MaxIntentsPerSchema>
>;
```

We intentionally avoid maintaining a reverse map of `IntentId → SchemaVersionId[]` to reduce runtime storage overhead,
since
this is primarily used for SDK tooling, though it could be added to an off-chain index.

----------

### 🧹 Design Simplification

The current runtime includes a top-level flag indicating whether any delegation exists between a user and provider. This
was used to shortcut delegation checks. In this model, that summary flag is dropped — its only purpose was to optimize
for the empty case. If need be, the design can be updated to preserve this structure.

This simplification removes complexity and keeps delegation logic self-contained.

## 11. **Governance and Publication Workflows** <a id="section_11"></a>

The publication of protocols, schemas, versions, and intents is tightly linked to governance controls and
administrative authority.

### 📜 Current Model

- All schema publication requires governance approval.
- Schema protocols are effectively ungoverned string-to-ID mappings.
- There is no enforcement of namespace or protocol ownership (other than manual review by governance).

### ✅ Proposed Model

| Asset Type    | Creation                                    | Update                            | Ownership              | Delegation    |
|---------------|---------------------------------------------|-----------------------------------|------------------------|---------------|
| Protocol      | Governance                                  | N/A                               | MSA                    | (Future work) |
| Schema        | Governance                                  | Minor versions via protocol owner | Protocol owner         | (Future work) |
| SchemaVersion | Published by namespace owner under protocol | Immutable                         | Associated with schema | N/A           |
| Intent        | Governance                                  | Immutable                         | None                   | N/A           |

### 🛠 Trusted Publishers (Future)

We may support a delegation model that allows namespace owners to appoint trusted publisher entities (MSAs or Accounts)
to publish new schema versions or schemas on their behalf.

- Only applicable for minor version updates
- May be tracked on-chain via `TrustedPublishers<NamespaceId>`
- Enables ecosystem flexibility without weakening governance integrity

### ⚠️ Major Version Governance Constraint

To avoid governance overload and approval ambiguity:

- Namespace owners may only submit **minor** schema versions under existing schemas
- Any **material semantic change** (e.g., change of associated intent, incompatible schema shape) must be published as a
  new schema, and thus approved via governance

While it's difficult to define a strict on-chain method for determining "minor" vs. "major" updates, the system assumes
publishers will self-classify responsibly. A few non-exhaustive rules might include:

- Changing the list of implemented intents = Major _(could be checked on-chain)_
- Incompatible schema format changes = Major
- Adding fields with privacy implications = Major
- Adding or removing fields with no privacy implications = Minor

This encourages transparency and long-term stability of protocols while supporting iterative development.

## 12. **API Identifier Handling and Resolution** <a id="section_12"></a>

The choice of identifiers in runtime and SDK APIs has significant implications for both usability and performance.
Historically, Frequency APIs used `SchemaVersionId` for delegation and data interpretation. The proposed model must
account for
schemas and intents, while maintaining runtime efficiency.

### 🔢 Preferred On-Chain Identifiers

On-chain APIs and runtime logic should rely exclusively on numeric identifiers:

- `SchemaVersionId`: Immutable and used to validate data structure
- `SchemaId`: Used for schema-level delegation
- `IntentId`: Used for intent-level delegation

String identifiers such as protocol names, schema names, or intent names should never be parsed on-chain. These must
be resolved by SDKs and off-chain services (e.g., the Frequency Gateway).

### 🧠 Resolution Responsibility

Clients and SDKs must perform resolution of strings to IDs before invoking extrinsics. For example:

- Resolve `"dsnp.broadcast"` → `(ProtocolId, SchemaId)`
- Resolve `"message.publish"` → `IntentId`
- Resolve `"dsnp.message.publish"` → `(IntentId, ProtocolId)`

### 📬 Extrinsic Parameters

Three possible design patterns:

1. **Submit by SchemaId only (Legacy/Compatible):**

    - Fast, direct lookup
    - Requires on-chain schema metadata to determine intent/protocol for validation
    - May require multi-delegation validation (intent or schema)

2. **Submit by SchemaVersionId + IntentId (Explicit):**

    - Removes ambiguity when multiple intents are mapped
    - Enables precision delegation checks
    - Slightly more complex client integration

3. **Submit by IntentId only (No Schema):**

    - Enables non-schema-based actions (e.g., future custom actions)
    - Runtime must validate delegation directly

Note, it is not practical to pass a `SchemaId` as a parameter at this time, unless at such time as delegation by
specific `SchemaVersionId` is fully removed, and delegation by `IntentId` is not implemented.

Additionally, it is possible that future use cases will arise that make use of intents-based delegation _in the absence
of any schema at all_, in which case extrinsics implementing this model would simply require an `IntentId` for
delegation validation.

### 🎯 Developer Experience

- APIs should remain backward compatible with `SchemaVersionId`-based delegation.
- Whether to expose protocol- or intent-based extrinsics directly is an open question.
- SDKs such as the Frequency Gateway may offer string-based input and resolve IDs internally.

## 13. **Migration Strategy** <a id="section_13"></a>

Transitioning from the current schema version + delegation system to the new schema + intent model must be handled
carefully
to preserve backward compatibility while enabling the new architecture to roll out incrementally.

### 🧩 Legacy Support

- Legacy `SchemaVersionId`-based delegations will continue to be supported for existing providers
- Existing data using older schema versions remains valid
- Providers can continue operating without changes while the new system is phased in

### 🚀 Protocol Bootstrapping

- Governance may backfill `SchemaId`s and `ProtocolId`s for existing DSNP schemas
- Each legacy schema version will be added to its corresponding schema version array
- Historical schema versions will be placed in schema version order

### 🔁 Delegation Migration

- Providers can request users re-authorize using `SchemaId` or `IntentId` delegations at their discretion
- SDK tooling may provide utilities to convert legacy schema delegations into schema or intent equivalents

### 🛑 Sunset of SchemaVersion-Based Delegation (Future)

Once adoption of intent/protocol delegation is widespread:

- Chain governance may disable new schema version-based delegations
- Existing schema version-based delegations may be invalidated at a future block height
- Tooling will issue deprecation warnings ahead of this change

This strategy ensures a smooth, opt-in path to the new system while preserving all critical functionality during the
transition period.

## 14. **Open Questions and Next Steps** <a id="section_14"></a>

Several open questions remain that may influence the final implementation strategy and runtime behavior. These are left
intentionally unresolved to guide future design discussions within the development and governance communities.

### ❓ Open Questions

- Should protocol and schema ownership be bound to **MSAs** or to **raw public keys**?

- Many of the aspects of ownership/control/governance overlap with ENS-type functionality; should we implement in
  isolation, or separate out into a partial or full ENS implmentation?

- Should delegation resolution allow both **schema** and **intent** types simultaneously, or require mutual
  exclusivity? Or, should only one of them be implemented at all?

- Will we support **extrinsics** that use `IntentId` without referring to a `SchemaVersionId`?

- How should fallback resolution be handled when multiple intents are associated with a single schema?

- Will a `TrustedPublisher` model be implemented soon, and if so, what will its scope of authority include?

- What developer-facing tools will be needed to ease migration and debugging?

- Will schema deprecation or version invalidation ever be supported, or are all published schemas permanently
  active?

### 🧭 Next Steps

1. **Internal review** of this proposal by Frequency core contributors

2. **Discussion of delegation scope models** and confirmation of preferred resolution pattern

3. **Engineering spike** into schema/intents/namespace runtime storage cost and migration

4. **API/SDK prototype** demonstrating resolution and delegation lookup UX

5. **Drafting of runtime implementation plan** and migration support utilities

6. **Call for feedback** from developers, partners, and governance stakeholders

This document serves as a foundation for these efforts and should evolve alongside prototyping and implementation
planning.
