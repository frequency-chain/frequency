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

//! Runtime API definition for [Messages](../pallet_messages/index.html)
//!
//! This api must be implemented by the node runtime.
//! Runtime APIs Provide:
//! - An interface between the runtime and Custom RPCs.
//! - Runtime interfaces for end users beyond just State Queries

use common_primitives::{messages::*, node::BlockNumber, schema::*};
extern crate alloc;
use alloc::vec::Vec;

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime files (the `runtime` folder)
sp_api::decl_runtime_apis! {

	/// Runtime Version for Messages
	/// - MUST be incremented if anything changes
	/// - See: https://paritytech.github.io/polkadot/doc/polkadot_primitives/runtime_api/index.html
	#[api_version(1)]

	/// Runtime APIs for [Messages](../pallet_messages/index.html)
	pub trait MessagesRuntimeApi
	{
		/// Retrieve the messages for a particular schema and block number
		fn get_messages_by_schema_and_block(schema_id: SchemaId, schema_payload_location: PayloadLocation, block_number: BlockNumber) ->
			Vec<MessageResponse>;

		/// Retrieve a schema by id
		fn get_schema_by_id(schema_id: SchemaId) -> Option<SchemaResponse>;
	}
}
