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

//! Runtime API definition for [Messages](pallet-messages)
//!
//! This api must be implemented by the node runtime.
//! Runtime APIs Provide:
//! - An interface between the runtime and Custom RPCs.
//! - Runtime interfaces for end users beyond just State Queries

use codec::Codec;
use common_primitives::{messages::*, schema::*};
use frame_support::inherent::Vec;

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime files (the `runtime` folder)
sp_api::decl_runtime_apis! {

	/// Runtime APIs for [Messages](pallet-messages)
	pub trait MessagesRuntimeApi<BlockNumber> where
		BlockNumber: Codec,
	{
		/// Retrieve the messages for a particular schema and block number
		fn get_messages_by_schema_and_block(schema_id: SchemaId, schema_payload_location: PayloadLocation, block_number: BlockNumber) ->
			Vec<MessageResponse<BlockNumber>>;

		/// Retrieve a schema by id
		fn get_schema_by_id(schema_id: SchemaId) -> Option<SchemaResponse>;
	}
}
