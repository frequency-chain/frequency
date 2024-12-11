#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]
#![allow(rustdoc::bare_urls)]
// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

//! Runtime API definition for [Schemas](../pallet_schemas/index.html)
//!
//! This api must be implemented by the node runtime.
//! Runtime APIs Provide:
//! - An interface between the runtime and Custom RPCs.
//! - Runtime interfaces for end users beyond just State Queries

use common_primitives::schema::*;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {

	/// Runtime Version for Schemas
	/// - MUST be incremented if anything changes
	/// - See: https://paritytech.github.io/polkadot/doc/polkadot_primitives/runtime_api/index.html
	#[api_version(2)]

	/// Runtime API definition for [Schemas](../pallet_schemas/index.html)
	pub trait SchemasRuntimeApi
	{
		/// Fetch the schema by id
		fn get_by_schema_id(schema_id: SchemaId) -> Option<SchemaResponse>;
		/// Fetch the schema versions by name
		fn get_schema_versions_by_name(schema_name: Vec<u8>) -> Option<Vec<SchemaVersionResponse>>;
	}
}
