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

/// SignedExtenions for pallet_democracy
pub mod democracy;
mod mock;
mod tests;
