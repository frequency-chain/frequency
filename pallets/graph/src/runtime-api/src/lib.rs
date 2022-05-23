#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use common_primitives::msa::MessageSenderId;
use frame_support::dispatch::DispatchError;
use sp_std::vec::Vec;

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime file (the `runtime/src/lib.rs`)
sp_api::decl_runtime_apis! {
	pub trait GraphApi where
	{
		fn get_following_list_public(static_id: MessageSenderId) -> Result<Vec<MessageSenderId>, DispatchError>;
	}
}
