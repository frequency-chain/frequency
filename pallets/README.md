# Creating New Pallets

## Add Documentation Lints

In `src/lib.rs` add these lines:

```rust
// Strong Documentation Lints
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::missing_crate_level_docs)]
#![deny(rustdoc::invalid_codeblock_attributes)]
```
