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

//! Runtime API definition for [Stateful  Storage](../pallet_stateful_storage/index.html)
//!
//! This api must be implemented by the node runtime.
//! Runtime APIs Provide:
//! - An interface between the runtime and Custom RPCs.
//! - Runtime interfaces for end users beyond just State Queries

use common_primitives::{
	msa::MessageSourceId,
	schema::SchemaId,
	stateful_storage::{ItemizedStoragePageResponse, PaginatedStorageResponse},
};
use sp_runtime::DispatchError;
use sp_std::vec::Vec;

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime files (the `runtime` folder)
sp_api::decl_runtime_apis! {

	/// Runtime Version for Stateful Storage
	/// - MUST be incremented if anything changes
	/// - See: https://paritytech.github.io/polkadot/doc/polkadot_primitives/runtime_api/index.html
	#[api_version(1)]

	/// Runtime APIs for [Stateful Storage](../pallet_stateful_storage/index.html)
	pub trait StatefulStorageRuntimeApi
	{
		/// Retrieve the paginated storage for a particular msa and schema
		fn get_paginated_storage(msa_id: MessageSourceId, schema_id: SchemaId) -> Result<Vec<PaginatedStorageResponse>, DispatchError>;
		/// Retrieve the itemized storage for a particular msa and schema
		fn get_itemized_storage(msa_id: MessageSourceId, schema_id: SchemaId) -> Result<ItemizedStoragePageResponse, DispatchError>;
	}
}
