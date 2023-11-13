# On Chain Message Storage

## Context and Scope
The proposed feature consists of changes that is going to be one (or more) pallet(s) in runtime of a
Substrate based blockchain, and it will be used in all environments including production.

## Problem Statement
After introduction of **Proof of Validity** or **PoV** in runtime weights, all pallets should be
re-evaluated and refactored if necessary to minimize the usage of **PoV**. This is to ensure all
important operations are scalable.
This document tries to propose some changes on **Schemas** pallet to optimize the **PoV** size.

## Goals
- Minimizing Weights including **execution times** and **PoV** size.

## Proposal
Split Schemas into `SchemaInfo` and `payload` would allow lower **PoV** when verifying schema existence
or compatibility.

### Main Storage types
- **SchemaInfos**
    - _Type_: `StorageMap<SchemaId, SchemaInfo>`
    - _Purpose_: Main structure To store related properties of any schema
      index
- **SchemaPayloads**
    -  _Type_: `StorageMap<SchemaId, BoundedVec<u8>>`
    - _Purpose_: Stores the payload or model for each schema


### On Chain Structure
Following is a proposed data structure for storing schema information on chain.
```rust
pub struct SchemaInfo {
    /// The type of model (AvroBinary, Parquet, etc.)
    pub model_type: ModelType,
    /// The payload location
    pub payload_location: PayloadLocation,
    /// additional control settings for the schema
    pub settings: SchemaSettings,
}
```
### Expected PoV improvements
This PoV improvement would not affect extrinsic weights in this pallet, but it would directly affect any
pallet that is dependent on **Schemas** pallet. Some of these pallets are **Messages** and
**Stateful-Storage**. After these changes we are expecting see to see around 30-60KiB decrease in PoV
for `MaxEncodedLen` mode.
