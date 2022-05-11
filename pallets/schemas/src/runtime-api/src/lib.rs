#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use frame_support::dispatch::DispatchError;
use pallet_transaction_payment::FeeDetails;
use sp_runtime::traits::MaybeDisplay;

sp_api::decl_runtime_apis! {
	pub trait SchemasRuntimeApi<AccountId, Balance> where
	AccountId: codec::Codec,
	Balance: Codec + MaybeDisplay,
	{
		fn get_latest_schema_id() -> Result<u16, DispatchError>;
		fn query_fee_details(uxt: Block::Extrinsic, len: u32) -> FeeDetails<Balance>;
	}
}
