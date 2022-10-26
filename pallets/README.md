# Creating New Pallets

## Pallet Documentation Template

```rust
//! # Pallet Name
//! Super short description
//!
//! - [Configuration: `Config`](Config)
//! - [Extrenisics: `Call`](Call)
//! - [Runtime API: `PALLETRuntimeApi`](../pallet_PALLET_runtime_api/trait.PALLETRuntimeApi.html)
//! - [Custom RPC API: `PALLETApiServer`](../pallet_PALLET_rpc/trait.PALLETApiServer.html)
//! - [Event Enum: `Event`](Event)
//! - [Error Enum: `Error`](Error)
//!
//! ## Overview
//! What does this pallet do or provide?
//!
//! ## Terminology
//! - **Term Here:** definition
//! - **Term Here:** Some term duplication between pallets is fine
//!
//! ## Implementations
//! (Beyond the standard, if any)
//!
//! - [`Trait`](../remember_to_use_links/when_outside_the_pallet/trait.TRAIT.html)
//!
```

## Add Documentation Lints

In these files:
- `src/lib.rs`
- `src/runtime-api/src/lib.rs`
- `src/rpc/src/lib.rs`

Add these lines:

```rust
// Strong Documentation Lints
#![deny(
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::invalid_codeblock_attributes,
    missing_docs
)]
```
