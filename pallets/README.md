# Creating New Pallets

## Add Documentation Lints

In `src/lib.rs` add these lines:

```rust
// Strong Documentation Lints
#![deny(
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::invalid_codeblock_attributes
)]
```
