//! Types for the Schema Pallet
use codec::{Decode, Encode, MaxEncodedLen};
use common_primitives::schema::{ModelType, PayloadLocation, SchemaSettings};
use frame_support::{
	traits::{Get, StorageVersion},
	BoundedVec,
};
use scale_info::TypeInfo;
use sp_std::fmt::Debug;

/// Current storage version of the schemas pallet.
pub const SCHEMA_STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxModelSize))]
#[codec(mel_bound(MaxModelSize: MaxEncodedLen))]
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
	/// additional control settings for the schema
	pub settings: SchemaSettings,
}

#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen)]
/// A structure defining a Schema information (excluding the payload)
pub struct SchemaInfo {
	/// The type of model (AvroBinary, Parquet, etc.)
	pub model_type: ModelType,
	/// The payload location
	pub payload_location: PayloadLocation,
	/// additional control settings for the schema
	pub settings: SchemaSettings,
}
