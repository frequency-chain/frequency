#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use common_primitives::{msa::*, node::BlockNumber};
use frame_support::dispatch::DispatchError;
use sp_std::vec::Vec;

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime file (the `runtime/src/lib.rs`)
sp_api::decl_runtime_apis! {
	pub trait MsaApi<AccountId> where
		AccountId: Codec,
	{
		// *Temporarily Removed* until https://github.com/LibertyDSNP/frequency/issues/418 is completed
		// fn get_msa_keys(msa_id: MessageSourceId) ->	Result<Vec<KeyInfoResponse<AccountId>>, DispatchError>;

		fn has_delegation(delegator: Delegator, provider: Provider, block_number: Option<BlockNumber>) -> bool;

		fn get_granted_schemas_by_msa_id(delegator: Delegator, provider: Provider) -> Result<Option<Vec<SchemaId>>, DispatchError>;
	}
}
