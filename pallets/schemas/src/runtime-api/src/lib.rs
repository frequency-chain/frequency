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
extern crate alloc;
use alloc::vec::Vec;

sp_api::decl_runtime_apis! {

	/// Runtime Version for Schemas
	/// - See: https://paritytech.github.io/polkadot/doc/polkadot_primitives/runtime_api/index.html
	// TODO: When deprecated methods are removed, bump base version
	#[api_version(2)]

	/// Runtime API definition for [Schemas](../pallet_schemas/index.html)
	pub trait SchemasRuntimeApi
	{
		/// Fetch the schema by id
		// TODO: Deprecated method; keep in place until all RPC nodes have been upgraded to remove the custom RPC `get_by_schema_id`
		#[deprecated(note = "Please use get_schema_by_id")]
		fn get_by_schema_id(schema_id: SchemaId) -> Option<SchemaResponse>;

		/// Fetch the schema by id
		#[api_version(3)]
		fn get_schema_by_id(schema_id: SchemaId) -> Option<SchemaResponseV2>;

		/// Fetch the schema versions by name
		// TODO: Deprecated method; keep in place until all RPC nodes have been upgraded to remove the custom RPC `get_versions`
		#[deprecated(note = "Schemas no longer have names; use get_registered_entities_by_name instead")]
		fn get_schema_versions_by_name(schema_name: Vec<u8>) -> Option<Vec<SchemaVersionResponse>>;

		/// Fetch registered entity identifiers by name
		#[api_version(3)]
		fn get_registered_entities_by_name(name: Vec<u8>) -> Option<Vec<NameLookupResponse>>;

		/// Fetch the Intent by id
		#[api_version(3)]
		fn get_intent_by_id(intent_id: IntentId, include_schemas: bool) -> Option<IntentResponse>;

		/// Fetch the IntentGroup by id
		#[api_version(3)]
		fn get_intent_group_by_id(intent_group_id: IntentGroupId) -> Option<IntentGroupResponse>;
	}
}
