#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use common_primitives::{messages::*, schema::*};
use frame_support::inherent::Vec;

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime file (the `runtime/src/lib.rs`)
sp_api::decl_runtime_apis! {
	pub trait MessagesRuntimeApi<BlockNumber> where
		BlockNumber: Codec,
	{
		fn get_messages_by_schema_and_block(schema_id: SchemaId, schema_payload_location: PayloadLocation, block_number: BlockNumber) ->
			Vec<MessageResponse<BlockNumber>>;

		fn get_schema_by_id(schema_id: SchemaId) -> Option<SchemaResponse>;
	}
}
