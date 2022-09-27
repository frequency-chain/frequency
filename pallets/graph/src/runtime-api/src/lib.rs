#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use common_primitives::{graph::*, msa::*};
use frame_support::dispatch::DispatchError;
use sp_std::prelude::*;

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime file (the `runtime/src/lib.rs`)
sp_api::decl_runtime_apis! {
	pub trait GraphApi
	{
		fn get_private_graph(msa_id: MessageSourceId) ->
		Result<
			Vec<(GraphKey, PrivatePage)>,
			DispatchError,
		>;
	}
}
