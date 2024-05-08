# Creating New Pallets

## Pallet Documentation Template

- The pallet should have a `README.md` file in the root. See `README.template.md` for a template.
- The Readme file should strictly follow the standard as the contents may be used elsewhere.
- The Readme file _must only_ use full links so the links work where ever the content is used.
- Any additional technical notes for Frequency developers may be placed in the docs after the Readme is included.

The standard documentation header for `lib.rs`:

```rust
//! Super short description
//!
//! ## Quick Links
//! - [Configuration: `Config`](Config)
//! - [Extrinsics: `Call`](Call)
//! - [Runtime API: `PALLETRuntimeApi`](../pallet_PALLET_runtime_api/trait.PALLETRuntimeApi.html)
//! - [Custom RPC API: `PALLETApiServer`](../pallet_PALLET_rpc/trait.PALLETApiServer.html)
//! - [Event Enum: `Event`](Event)
//! - [Error Enum: `Error`](Error)
#![doc = include_str!("../README.md")]
//! Optional additional Rust developer documentation
```

Feel free to add additional Rust developer documentation after the Readme.

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
