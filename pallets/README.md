# Creating New Pallets

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
