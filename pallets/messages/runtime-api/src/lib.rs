#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use common_primitives::{messages::*, schema::*};
use frame_support::dispatch::DispatchError;

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime file (the `runtime/src/lib.rs`)
sp_api::decl_runtime_apis! {
	pub trait MessagesApi<AccountId, BlockNumber> where
		AccountId: Codec,
		BlockNumber: Codec,
	{
		fn get_messages_by_schema(schema_id: SchemaId,
			pagination: BlockPaginationRequest<BlockNumber>) ->
		Result<
			BlockPaginationResponse<BlockNumber, MessageResponse<AccountId, BlockNumber>>,
			DispatchError,
		>;
	}
}
