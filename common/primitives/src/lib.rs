//! # Frequency Primitives
//!
//! Primitives package contains many of the structs and trait implementations
//! for Pallets and utilities that need to be shared across packages

#![cfg_attr(not(feature = "std"), no_std)]
// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

/// Data structures for the generic use.
pub mod ds;
/// Structs and traits for the Messages pallet.
pub mod messages;
/// Structs and traits for the MSA pallet.
pub mod msa;
/// Node level primitives.
pub mod node;
/// Structs and traits for parquet
pub mod parquet;
/// Structs and traits for the Schema pallet
pub mod schema;
/// Structs and traits for the utility package.
pub mod utils;
