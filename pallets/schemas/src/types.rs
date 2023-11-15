//! Types for the Schema Pallet
use common_primitives::schema::{ModelType, PayloadLocation, SchemaSettings};
use frame_support::traits::StorageVersion;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_std::fmt::Debug;

/// Current storage version of the schemas pallet.
pub const SCHEMA_STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

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
