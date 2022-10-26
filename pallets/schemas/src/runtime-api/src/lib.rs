#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]
// Strong Documentation Lints
#![deny(
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs,
	rustdoc::invalid_codeblock_attributes,
	missing_docs
)]

//! Runtime API definition for [Schemas](pallet-schemas)
//!
//! This api must be implemented by the node runtime.
//! Runtime APIs Provide:
//! - An interface between the runtime and Custom RPCs.
//! - Runtime interfaces for end users beyond just State Queries

use common_primitives::schema::*;
use frame_support::dispatch::DispatchError;

sp_api::decl_runtime_apis! {
	/// Runtime API definition for [Schemas](pallet-schemas)
	pub trait SchemasRuntimeApi
	{
		/// Fetch the schema by id
		fn get_by_schema_id(schema_id: SchemaId) -> Result<Option<SchemaResponse>, DispatchError>;
	}
}
