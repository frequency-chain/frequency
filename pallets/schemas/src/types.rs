//! Types for the Schema Pallet
use common_primitives::schema::{ModelType, PayloadLocation};
use frame_support::{traits::Get, BoundedVec};
use scale_info::TypeInfo;
use common_primitives::parquet::ParquetModel;

use codec::{Decode, Encode, MaxEncodedLen};

#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen)]
/// A structure defining a Schema
pub struct Schema
{
	/// The type of model (AvroBinary, Parquet, etc.)
	pub model_type: ModelType,
	/// Defines the structure of the message payload using model_type
	pub model: ParquetModel,
	/// The payload location
	pub payload_location: PayloadLocation,
}
