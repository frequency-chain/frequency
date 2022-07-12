//! Types for the Schema Pallet
use common_primitives::schema::{ModelType, PayloadLocation};
use frame_support::{traits::Get, BoundedVec};
use scale_info::TypeInfo;
use common_primitives::parquet::ParquetModel;

use codec::{Decode, Encode, MaxEncodedLen};

#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxModelSize))]
/// A structure defining a Schema
pub struct Schema<MaxModelSize>
where
	MaxModelSize: Get<u32>,
{
	/// The type of model (AvroBinary, Parquet, etc.)
	pub model_type: ModelType,
	/// Defines the structure of the message payload using model_type
	pub model: BoundedVec<u8, MaxModelSize>,
	/// The payload location
	pub payload_location: PayloadLocation,
}
