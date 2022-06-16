#![cfg_attr(not(feature = "std"), no_std)]
// Strong Documentation Lints
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::missing_crate_level_docs)]
#![deny(rustdoc::invalid_codeblock_attributes)]

pub mod messages;
pub mod msa;
#[cfg(feature = "std")]
pub mod rpc;
pub mod schema;
pub mod utils;
