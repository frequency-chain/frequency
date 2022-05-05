#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use frame_support::dispatch::DispatchError;

sp_api::decl_runtime_apis! {
	pub trait SchemaRuntimeApi<AccountId> where
	AccountId: codec::Codec
	{
		fn get_latest_schema_id() -> Result<u16, DispatchError>;
	}
}
