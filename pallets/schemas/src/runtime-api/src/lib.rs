#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use common_primitives::schema::*;
use frame_support::dispatch::DispatchError;

sp_api::decl_runtime_apis! {
	pub trait SchemasRuntimeApi
	{
		fn get_latest_schema_id() -> Option<SchemaId>;
		fn get_by_schema_id(schema_id: SchemaId) -> Result<Option<SchemaResponse>, DispatchError>;
	}
}
