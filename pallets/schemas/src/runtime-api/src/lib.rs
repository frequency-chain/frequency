#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use common_primitives::schema::*;
use frame_support::{dispatch::DispatchError, weights::Weight};
use sp_runtime::traits::MaybeDisplay;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
	pub trait SchemasRuntimeApi<Balance> where
	Balance: Codec + MaybeDisplay,
	{
		fn get_latest_schema_id() -> Option<SchemaId>;
		fn calculate_schema_cost(schema: Vec<u8>) -> Weight;
		fn get_by_schema_id(schema_id: SchemaId) -> Result<Option<SchemaResponse>, DispatchError>;
	}
}
