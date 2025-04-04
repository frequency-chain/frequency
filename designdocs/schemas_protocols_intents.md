
# 📘 Schema Protocols and Intent-Based Delegation in Frequency

## 1. **Background and Motivation**

In the current implementation, schemas are registered with immutable numeric identifiers (`SchemaId`) and describe the layout and storage semantics (e.g., Avro/Parquet formats, on-chain/off-chain storage). These schema IDs are used as references by clients and runtime modules alike, particularly in the delegation system defined by the `msa` pallet.

Delegations currently allow a user to authorize a provider (e.g., an app or service) to act on their behalf, but this authorization is tightly bound to a specific `SchemaId`. This model has proven limiting in several ways:

-   **Coupling between schema versions and delegations**
-   **Schemas represent data format, not purpose**
-   **Lack of human-readable context**
-   **Rigid governance workflows**
-   **No concept of publisher authority**

These limitations have motivated a re-architecture of the schema and delegation systems to introduce the concepts of:

-   **Named schema protocols** with version tracking
-   **Structured publishing under namespaces**
-   **Intent-based delegation**
-   **More expressive APIs and storage models**

## 2. **Design Goals**

This section outlines the key objectives that guide the redesign of Frequency's schema and delegation architecture.

-   **Schema Immutability**
-   **Protocol-Based Versioning**
-   **Minimal Delegation Churn**
-   **Intent Separation**
-   **Scoped Delegations**
-   **Namespace Ownership and Governance**
-   **On-Chain Efficiency**

## 3. **Schema Protocols and Versioning**

Protocols are not just organizational tools—they enable meaningful delegation. In the new model, a protocol is a first-class on-chain entity that represents a named and versioned group of schemas. Every schema is assigned to a protocol, and new versions of a protocol simply add new schema IDs to the protocol's version history.

### 📌 Protocol Structure

-   `ProtocolId`: A unique numeric identifier for runtime efficiency
-   `NamespaceId`: Which namespace owns this protocol
-   `Name`: Human-readable protocol name (e.g., `broadcast`)
-   `Versions`: Ordered list of `SchemaId`s (e.g., v0 → v1 → v2)
-   Metadata: Optional description, links, etc.

### 🧬 Versioning Rules

-   Versions are stored in a monotonic, gapless array
-   A new schema ID is automatically assigned the next version
-   Older versions are preserved permanently
-   Delegations may apply to the protocol, not a specific schema ID
- _Question: can versions be deprecated?_

### 🧭 Governance Rules

-   New protocols must be approved by governance
-   Minor updates (new schema version) may be published by the namespace owner
-   Major updates require a new protocol (e.g., change in semantic intent)

## 4. **Namespace Ownership**

In order to provide organizational control, accountability, and publishing boundaries for schemas and intents, the protocol introduces the concept of **namespaces**.

A namespace (e.g., `dsnp`, `bsky`) serves as a root authority under which protocols and intents are registered. Namespaces allow developers and governance bodies to clearly define who may publish what, and to prevent namespace squatting or unauthorized use of shared protocol identifiers.

### 📛 Namespace Structure

Each namespace is a unique identifier, typically human-readable (e.g., `"dsnp"`), mapped to an on-chain `NamespaceId` (e.g., `NamespaceId = 2`).

-   Every protocol must be published under a namespace: `namespace.protocol`
-   Intents may also be published under a namespace _(open question, discussed below)_

### 👤 Ownership and Control

Namespaces are envisioned to be owned by an entity, but there is an open question as to whether this entity should be an MSA (i.e., a Provider) or a raw account (i.e., a public key). This decision will impact how access is managed, especially in governance and delegation flows.

If namespaces are owned by MSAs, they inherit support for delegation and signature validation via MSA structures. If they are owned by raw accounts, they align more directly with typical on-chain identity models and may simplify some access control paths.

Regardless of the final model, the owner of a namespace has exclusive authority to:
-   Publish protocols and schemas under the namespace
-   Register intents (namespaced or global) scoped to the namespace
-   Transfer namespace ownership (via governance or extrinsic)
-   Assign trusted publishers or revoke their rights _(future enhancement)_

### ⚙️ Namespace Governance

-   **Creation of a new namespace** must be approved via governance.
-   Once approved, a namespace is permanently registered and owned by an MSA.
-   Future governance tracks may support delegation or stewardship models (e.g., rotating maintainers, multisig groups).

### 🔐 Namespace-Based Security Boundaries

Namespaces define logical and administrative isolation between ecosystems. For example:
-   `dsnp.broadcast` is owned by the DSNP team
-   `bsky.profile` is controlled by Bluesky
-   No party may publish schemas into another's namespace without explicit permission

This structure provides a trust boundary that maps to real-world organizational authority.

## 5. **Delegation Semantics**

Delegation is the mechanism by which a user authorizes a provider to act on their behalf. Currently, this is limited to individual `SchemaId`s, but we propose expanding the delegation model to include one or more of the following:
-   Delegation by `SchemaId` (existing/legacy, for future deprecation)
-   Delegation by `ProtocolId` (implies all schema versions with the protocol)
-   Delegation by `IntentId` (e.g., “message.publish”), with optional namespace scoping

All three modes may coexist, allowing for graceful migration and flexible usage.

Each delegation will include:
```rust
pub struct DelegationInfo {
    pub entity: DelegationTarget,          // Schema, Protocol, or Intent(+Namespace)
    pub revoked_at: Option<BlockNumber>,   // Replaces `expiration` and `revoked`
    pub granted_at: BlockNumber,
}
```

And:
```rust
pub enum DelegationTarget {
    Schema(SchemaId),
    Protocol(ProtocolId),
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

## 6. **Intent Registration & Delegation Models**

Intents represent abstract actions or operations that a provider may be authorized to perform on behalf of a user. While schemas define how data is structured, intents define what that data _means_ or how it is _used_.

For example, `message.publish` might correspond to submitting a post to a public timeline, regardless of whether it's encoded using `broadcast`, `reply`, or `reshare` schemas.

We explore three models for intent registration and delegation:

### 🔹 Model A: Chain-Global Intents
-   Intents are globally registered, usable across namespaces
-   A single `IntentId` (e.g., `message.publish`) applies to all schemas that implement it

**Pros**:
-   Simple
-   Reusable across protocols
-   Easy for SDKs to reference

**Cons**:
-   No isolation between ecosystems
-   Potential name conflicts or governance tension

### 🔸 Model B: Namespaced Intents

-   Intents are registered under a specific namespace (e.g., `dsnp.message.publish`)
-   Each namespace manages its own intent catalog

**Pros**:
-   Strong isolation
-   Prevents cross-ecosystem collisions

**Cons**:
-   Duplicative
-   Harder for SDKs to unify

### 🔷 Model C: Hybrid (Recommended)

-   Intents are globally registered (e.g., `message.publish`)
-   Delegations may scope usage to specific namespaces (e.g., `message.publish` for `dsnp` only)

**Pros**:
-   Shared meaning
-   Namespace-level trust boundary
-   Cleanest balance of safety + reuse

**Cons**:
-   Slightly more complex resolution logic
-   Requires schema metadata to track which intents are implemented

### 🚫 Considered and Dismissed: Hybrid 2 — Both Global and Namespaced Intents
-   Intents may be either global or namespaced
-   Global intents require governance approval
-   Namespaced intents may be published by the namespace owner

**Issue:** This model creates potential ambiguity and collision between global and namespaced intents sharing the same base name (e.g., both `message.publish` and `dsnp.message.publish`). It also complicates the future introduction of global intents if a namespaced version already exists.

Since the only expected use case for namespaced intents is to fill the gap where no global intent exists, the cleaner long-term solution is to require governance-approved global-only intents, while using delegation scoping to preserve control.

## 7. **Intent Registry Design**

The intent registry is an on-chain mapping of supported intents, including associated metadata and governance control.

### 🗂 Intent Metadata

Each intent has the following attributes:
-   `IntentId`: Compact numeric ID used on-chain
-   `Name`: Human-readable name (e.g., `message.publish`)
-   `Description`: Optional developer-facing documentation
-   `Link`: Optional link to spec or docs
-   `RegisteredBy`: MSA that registered the intent
-   `RegisteredAt`: Block number
- _Question: can intents be deprecated?_

Intents are immutable once registered. Only governance can publish new global intents.

**Question:** should the intent registry be multi-level (ie `message` -> [`publish`, `react`])?
**Pros:**
* Makes it possible to delegate to `message.*`

**Cons:**
* Increased complexity
* Storage and runtime delegation resolution costs?

### 📥 Schema → Intents Mapping

Schemas declare the list of intents they support in their metadata. This association is stored on-chain and must be fixed at the time of schema publication. This allows the runtime to evaluate whether a provider acting under a certain intent has been delegated for the schema being used.
```rust
pub type SchemaIntents<T: Config> = StorageMap<
    _, Blake2_128Concat, SchemaId,
    BoundedVec<IntentId, MaxIntentsPerSchema>
>;
```
This mapping supports intent-level delegation by ensuring that intent-level delegation can be efficiently resolved from a `SchemaId`.

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
-   Global intents: Intents are only publishable by governance.
-   Namespaced intents: namespace owners may register intents

## 8. **Authorization Resolution Models**

This section outlines how delegation is verified at the time of an extrinsic call or message submission. In the new model, the runtime may evaluate permissions based on either a specific `SchemaId`, or based on a broader context such as `IntentId` or `ProtocolId`.

### 🔁 Delegation Type Considerations

Runtime costs are probably the deciding factor in whether to support protocol-based delegation, intents-based delegation, or both. The resolution logic depends on the type of delegation being supported:

-   **Protocol-based delegation**:
    -   No additional information is required.
    -   Every `SchemaId` maps to a single `ProtocolId`, so the runtime can directly validate whether the provider is authorized under the protocol.

-   **Intent-based delegation**:
    -   If only a `SchemaId` is provided, the runtime will retrieve the list of declared intents.
    -   The runtime must then determine whether **any** or **all** of the schema’s intents must be delegated to the provider. This policy must be clearly defined and consistent across use cases.
    -   If an `IntentId` is explicitly provided, the runtime can perform a targeted check.

-   **Schema-based delegation**:
    -   Supported for backward compatibility. Schema-level delegations override intent and protocol resolution when present.


In practice, the resolution model may need to check **multiple types of delegation** in parallel. For example:

-   If both intent-based and protocol-based delegations are supported, the runtime may validate a call as authorized if **either** a valid intent **or** protocol delegation is found.
-   The delegation resolution policy (e.g., precedence, fallback order, or required combinations) must be explicitly defined by the runtime and documented for developers.

This multi-path resolution allows greater flexibility, but requires care to avoid ambiguity and ensure consistent behavior across different APIs and use cases.

### 🧪 Additional Considerations

-   In the future, extrinsics may support intent-based authorization **without referencing a schema** at all. These would rely solely on explicit `IntentId` + optional namespace context.
-   If a schema supports multiple intents, fallback rules and audit logic must be clear.
-   SDKs should provide helpers for intent resolution and delegation explanation.

## 9. **On-Chain Data Structures**

This section outlines the proposed on-chain data structures to support schema protocols, intents, delegations, and namespace ownership in the redesigned Frequency architecture.

### 🧱 Design Principles

-   On-chain runtime operations should only work with compact numeric identifiers (`SchemaId`, `ProtocolId`, `IntentId`, `NamespaceId`)
-   Human-readable names must be resolved by the client prior to calling extrinsics
-   Delegations are unified under a single structure, regardless of type
-   Minimize storage duplication and unnecessary indices

----------

### 🧩 Delegation Structures
```rust
pub enum DelegationTarget {
    Schema(SchemaId),
    Protocol(ProtocolId),
    Intent { id: IntentId, namespace: Option<NamespaceId> },
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

This structure handles all types of delegations (schema, protocol, intent) uniformly. Legacy `SchemaId` delegation remains supported, with protocol and intent-based delegation layered on top.

----------

### 🏷 Protocols and Namespaces
```rust
pub type NamespaceInfos<T> = StorageMap<
    _, Blake2_128Concat, NamespaceId, NamespaceInfo
>;

// Question: should we have a map of NamespaceId => Owner

pub type NamespaceProtocols<T> = StorageMap<
    _, Blake2_128Concat, NamespaceId,
    BoundedVec<ProtocolId, MaxProtocolsPerNamespace>
>;

pub type ProtocolVersions<T> = StorageMap<
    _, Blake2_128Concat, ProtocolId,
    BoundedVec<SchemaId, MaxVersionsPerProtocol>
>;

// Useful for efficient reverse-map of Schema to Protocol
// needed for checking protocol delegations
pub type SchemaProtocols<T> = StorageMap<
    _, Blake2_128Concat, SchemaId, ProtocolId
>;
```

Note: `ProtocolIds` are only referenced by numeric ID. Clients must resolve names via off-chain lookup using mappings from `NamespaceId + name → ProtocolId`.

----------

### 🧠 Intent Registry

```rust
pub type Intents<T> = StorageMap<
    _, Blake2_128Concat, IntentId, IntentMetadata
>;

// This info is contained in the SchemaInfo metadata, but
// a separate map helps with efficient lookup for intent-based delegation
pub type SchemaIntents<T> = StorageMap<
    _, Blake2_128Concat, SchemaId,
    BoundedVec<IntentId, MaxIntentsPerSchema>
>;
```

We intentionally avoid maintaining a reverse map of `IntentId → SchemaId[]` to reduce runtime storage overhead, since this is primarily used for SDK tooling, though it could be added to an off-chain index.

----------

### 🧹 Design Simplification

The current runtime includes a top-level flag indicating whether any delegation exists between a user and provider. This was used to shortcut delegation checks. In this model, that summary flag is dropped — its only purpose was to optimize for the empty case. If need be, the design can be updated to preserve this structure.

This simplification removes complexity and keeps delegation logic self-contained.

## 10. **Governance and Publication Workflows**

The publication of schemas, protocols, namespaces, and intents is tightly linked to governance controls and administrative authority.

### 📜 Current Model

-   All schema publication requires governance approval.
-   Schema protocols are effectively ungoverned string-to-ID mappings.
-   There is no enforcement of namespace or protocol ownership (other than manual review by governance).

### ✅ Proposed Model

| Asset Type | Creation | Update | Ownership | Delegation |
| ------------  | ---------- | --------- | ------------- | ------- |
| Namespace | Governance | N/A | MSA | (Future work) |
| Protocol | Governance |Minor versions via namespace owner | Namespace owner |(Future work) |
| Schema | Published by namespace owner under protocol | Immutable | Associated with protocol | N/A |
| Intent | Governance | Immutable | None | N/A |

### 🛠 Trusted Publishers (Future)

We may support a delegation model that allows namespace owners to appoint trusted publisher entities (MSAs or Accounts) to publish new schema versions or protocols on their behalf.

-   Only applicable for minor version updates
-   May be tracked on-chain via `TrustedPublishers<NamespaceId>`
-   Enables ecosystem flexibility without weakening governance integrity

### ⚠️ Major Version Governance Constraint

To avoid governance overload and approval ambiguity:

-   Namespace owners may only submit **minor** schema versions under existing protocols
-   Any **material semantic change** (e.g., change of associated intent, incompatible schema shape) must be published as a new protocol, and thus approved via governance

While it's difficult to define a strict on-chain method for determining "minor" vs. "major" updates, the system assumes publishers will self-classify responsibly. A few non-exhaustive rules might include:

-   Changing the list of implemented intents = Major _(could be checked on-chain)_
-   Incompatible schema format changes = Major
-   Adding fields with privacy implications = Major
-   Adding or removing fields with no privacy implications = Minor

This encourages transparency and long-term stability of protocols while supporting iterative development.

## 11. **API Identifier Handling and Resolution**

The choice of identifiers in runtime and SDK APIs has significant implications for both usability and performance. Historically, Frequency APIs used `SchemaId` for delegation and data interpretation. The proposed model must account for protocols and intents, while maintaining runtime efficiency.

### 🔢 Preferred On-Chain Identifiers

On-chain APIs and runtime logic should rely exclusively on numeric identifiers:

-   `SchemaId`: Immutable and used to validate data structure
-   `ProtocolId`: Used for protocol-level delegation
-   `IntentId`: Used for intent-level delegation

String identifiers such as namespace names, protocol names, or intent names should never be parsed on-chain. These must be resolved by SDKs and off-chain services (e.g., the Frequency Gateway).

### 🧠 Resolution Responsibility

Clients and SDKs must perform resolution of strings to IDs before invoking extrinsics. For example:

-   Resolve `"dsnp.broadcast"` → `(NamespaceId, ProtocolId)`
-   Resolve `"message.publish"` → `IntentId`
-   Resolve `"dsnp.message.publish"` → `(IntentId, NamespaceId)`

### 📬 Extrinsic Parameters

Three possible design patterns:

1.  **Submit by SchemaId only (Legacy/Compatible):**

    -   Fast, direct lookup
    -   Requires on-chain schema metadata to determine intent/protocol for validation
    -   May require multi-delegation validation (intent or protocol)

2.  **Submit by SchemaId + IntentId (Explicit):**

    -   Removes ambiguity when multiple intents are mapped
    -   Enables precision delegation checks
    -   Slightly more complex client integration

3.  **Submit by IntentId only (No Schema):**

    -   Enables non-schema-based actions (e.g., future custom actions)
    -   Runtime must validate delegation directly

Note, it is not practical to pass a `ProtocolId` as a parameter at this time, unless at such time as delegation by specific `SchemaId` is fully removed, and delegation by `IntentId` is not implemented.

Additionally, it is possible that future use cases will arise that make use of intents-based delegation _in the absence of any schema at all_, in which case extrinsics implementing this model would simply require an `IntentId` for delegation validation.

### 🎯 Developer Experience

-   APIs should remain backward compatible with `SchemaId`-based delegation.
-   Whether to expose protocol- or intent-based extrinsics directly is an open question.
-   SDKs such as the Frequency Gateway may offer string-based input and resolve IDs internally.

## 12. **Migration Strategy**

Transitioning from the current schema + delegation system to the new protocol + intent model must be handled carefully to preserve backward compatibility while enabling the new architecture to roll out incrementally.

### 🧩 Legacy Support

-   Legacy `SchemaId`-based delegations will continue to be supported for existing providers
-   Existing data using older schema versions remains valid
-   Providers can continue operating without changes while the new system is phased in

### 🚀 Protocol Bootstrapping

-   Governance may backfill `ProtocolId`s and `NamespaceId`s for existing DSNP schemas
-   Each legacy schema will be added to its corresponding protocol version array
-   Historical schemas will be placed in protocol version order


### 🔁 Delegation Migration

-   Providers can request users re-authorize using `ProtocolId` or `IntentId` delegations at their discretion
-   SDK tooling may provide utilities to convert legacy schema delegations into protocol or intent equivalents

### 🛑 Sunset of Schema-Based Delegation (Future)

Once adoption of intent/protocol delegation is widespread:

-   Chain governance may disable new schema-based delegations
-   Existing schema-based delegations may be invalidated at a future block height
-   Tooling will issue deprecation warnings ahead of this change

This strategy ensures a smooth, opt-in path to the new system while preserving all critical functionality during the transition period.

## 13. **Open Questions and Next Steps**

Several open questions remain that may influence the final implementation strategy and runtime behavior. These are left intentionally unresolved to guide future design discussions within the development and governance communities.

### ❓ Open Questions

-   Should namespace and protocol ownership be bound to **MSAs** or to **raw public keys**?

-   Should delegation resolution allow both **protocol** and **intent** types simultaneously, or require mutual exclusivity? Or, should only one of them be implemented at all?

-   Will we support **extrinsics** that use `IntentId` without referring to a `SchemaId`?

-   How should fallback resolution be handled when multiple intents are associated with a single schema?

-   Will a `TrustedPublisher` model be implemented soon, and if so, what will its scope of authority include?

-   What developer-facing tools will be needed to ease migration and debugging?

-   Will protocol deprecation or version invalidation ever be supported, or are all published protocols permanently active?


### 🧭 Next Steps

1.  **Internal review** of this proposal by Frequency core contributors

2.  **Discussion of delegation scope models** and confirmation of preferred resolution pattern

3.  **Engineering spike** into schema/intents/namespace runtime storage cost and migration

4.  **API/SDK prototype** demonstrating resolution and delegation lookup UX

5.  **Drafting of runtime implementation plan** and migration support utilities

6.  **Call for feedback** from developers, partners, and governance stakeholders


This document serves as a foundation for these efforts and should evolve alongside prototyping and implementation planning.
