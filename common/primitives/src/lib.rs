#![cfg_attr(not(feature = "std"), no_std)]
// Strong Documentation Lints
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::missing_crate_level_docs)]
#![deny(rustdoc::invalid_codeblock_attributes)]

pub mod messages;
/// export msa primitive types.
pub mod msa;
#[cfg(feature = "std")]
/// export rpc primitive types.
pub mod rpc;
/// export schema primitive types.
pub mod schema;
/// export utils primitive types.
pub mod utils;
