use codec::{Decode, Encode};
use common_primitives::schema::{SchemaId, SchemaResponse};
use frame_support::{traits::Get, BoundedVec};
use scale_info::TypeInfo;
use sp_std::prelude::*;

#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
#[scale_info(skip_type_params(MaxDataSize))]
pub struct Schema<MaxDataSize>
where
	MaxDataSize: Get<u32> + Clone,
{
	pub data: BoundedVec<u8, MaxDataSize>,
}

impl<MaxDataSize> Schema<MaxDataSize>
where
	MaxDataSize: Get<u32> + Clone,
{
	pub fn map_to_response(&self, schema_id: SchemaId) -> SchemaResponse {
		SchemaResponse { schema_id, data: self.data.clone().into_inner() }
	}
}
