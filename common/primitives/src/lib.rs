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
/// Types for the Handles pallet
pub mod handles;
/// macros
pub mod macros;
/// Structs and traits for the Messages pallet.
pub mod messages;
/// Structs and traits for the MSA pallet.
pub mod msa;
/// Node level primitives.
pub mod node;
/// Structs and traits for parquet
pub mod parquet;
/// Structs and traits for better RPCs
pub mod rpc;
/// Structs and traits for the Schema pallet
pub mod schema;
/// Types for the Stateful Storage pallet
pub mod stateful_storage;
/// Structs and traits for the utility package.
pub mod utils;

/// Structs and traits for the Capacity pallet.
pub mod capacity;

/// Constants and types for offchain data
pub mod offchain;

#[cfg(feature = "runtime-benchmarks")]
/// Benchmarking helper trait
pub mod benchmarks;

/// Signature support for ethereum
pub mod signatures;
