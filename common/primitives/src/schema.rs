extern crate alloc;
use crate::impl_codec_bitflags;
#[cfg(feature = "std")]
use crate::utils;
use alloc::{vec, vec::Vec};
use enumflags2::{bitflags, BitFlags};
use parity_scale_codec::{Decode, DecodeWithMemTracking, Encode, EncodeLike, MaxEncodedLen};
use scale_info::{build::Fields, meta_type, Path, Type, TypeInfo, TypeParameter};
use serde::{Deserialize, Serialize};
use sp_runtime::RuntimeDebug;
#[cfg(feature = "std")]
use utils::*;

/// Schema Id is the unique identifier for a Schema
pub type SchemaId = u16;

/// Schema version number
pub type SchemaVersion = u8;

/// Types of modeling in which a message payload may be defined
#[derive(
	Copy,
	Clone,
	Encode,
	Decode,
	DecodeWithMemTracking,
	PartialEq,
	Debug,
	TypeInfo,
	Eq,
	MaxEncodedLen,
	Serialize,
	Deserialize,
)]
pub enum ModelType {
	/// Message payload modeled with Apache Avro: <https://avro.apache.org/docs/current/spec.html>
	AvroBinary,
	/// Message payload modeled with Apache Parquet: <https://parquet.apache.org/>
	Parquet,
}

/// Types of payload locations
#[derive(
	Copy,
	Clone,
	Encode,
	Decode,
	DecodeWithMemTracking,
	PartialEq,
	Debug,
	TypeInfo,
	Eq,
	MaxEncodedLen,
	Serialize,
	Deserialize,
)]
pub enum PayloadLocation {
	/// Message payload is located on chain
	OnChain,
	/// Message payload is located on IPFS
	IPFS,
	/// Itemized payload location for onchain storage in itemized form
	Itemized,
	/// Paginated payload location for onchain storage in paginated form
	Paginated,
}

/// Support for up to 16 user-enabled features on a collection.
#[bitflags]
#[repr(u16)]
#[derive(
	Copy,
	Clone,
	RuntimeDebug,
	PartialEq,
	Eq,
	Encode,
	Decode,
	DecodeWithMemTracking,
	MaxEncodedLen,
	TypeInfo,
	Serialize,
	Deserialize,
)]
pub enum SchemaSetting {
	/// Schema setting to enforce append-only behavior on payload.
	/// Applied to schemas of type `PayloadLocation::Itemized`.
	AppendOnly,
	/// Schema may enforce signature requirement on payload.
	/// Applied to schemas of type `PayloadLocation::Itemized` or `PayloadLocation::Paginated`.
	SignatureRequired,
}

/// Wrapper type for `BitFlags<SchemaSetting>` that implements `Codec`.
#[derive(Clone, Copy, PartialEq, Eq, Default, RuntimeDebug)]
pub struct SchemaSettings(pub BitFlags<SchemaSetting>);

/// RPC Response form for a Schema
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct SchemaResponse {
	/// The unique identifier for this Schema
	pub schema_id: SchemaId,
	/// The data that represents how this schema is structured
	#[cfg_attr(feature = "std", serde(with = "as_string"))]
	pub model: Vec<u8>,
	/// The model format type for how the schema model is represented
	pub model_type: ModelType,
	/// The payload location
	pub payload_location: PayloadLocation,
	/// grants for the schema
	pub settings: Vec<SchemaSetting>,
}

/// RPC Response form for a Schema Info
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct SchemaInfoResponse {
	/// The unique identifier for this Schema
	pub schema_id: SchemaId,
	/// The model format type for how the schema model is represented
	pub model_type: ModelType,
	/// The payload location
	pub payload_location: PayloadLocation,
	/// grants for the schema
	pub settings: Vec<SchemaSetting>,
}

/// This allows other pallets to resolve Schema information. With generic SchemaId
pub trait SchemaProvider<SchemaId> {
	/// Gets the Schema details associated with this `SchemaId` if any
	fn get_schema_by_id(schema_id: SchemaId) -> Option<SchemaResponse>;

	/// Gets the Schema Info associated with this `SchemaId` if any
	fn get_schema_info_by_id(schema_id: SchemaId) -> Option<SchemaInfoResponse>;
}

/// This allows other Pallets to check validity of schema ids.
pub trait SchemaValidator<SchemaId> {
	/// Checks that a collection of SchemaIds are all valid
	fn are_all_schema_ids_valid(schema_ids: &[SchemaId]) -> bool;

	/// Set the schema counter for testing purposes.
	#[cfg(any(feature = "std", feature = "runtime-benchmarks", test))]
	fn set_schema_count(n: SchemaId);
}

impl SchemaSettings {
	/// Returns new SchemaSettings with all settings disabled
	pub fn all_disabled() -> Self {
		Self(BitFlags::EMPTY)
	}
	/// Get all setting enabled
	pub fn get_enabled(&self) -> BitFlags<SchemaSetting> {
		self.0
	}
	/// Check if a setting is enabled
	pub fn is_enabled(&self, grant: SchemaSetting) -> bool {
		self.0.contains(grant)
	}
	/// Enable a setting
	pub fn set(&mut self, grant: SchemaSetting) {
		self.0.insert(grant)
	}
	/// Copy the settings from a BitFlags
	pub fn from(settings: BitFlags<SchemaSetting>) -> Self {
		Self(settings)
	}
}
impl_codec_bitflags!(SchemaSettings, u16, SchemaSetting);

/// RPC Response from a schema name query
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct SchemaVersionResponse {
	/// Schema name in following format: namespace.descriptor
	#[cfg_attr(feature = "std", serde(with = "as_string"))]
	pub schema_name: Vec<u8>,
	/// The version for this schema
	pub schema_version: SchemaVersion,
	/// The unique identifier for this Schema
	pub schema_id: SchemaId,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn schema_settings_when_disabled_has_no_enabled() {
		let settings = SchemaSettings::all_disabled();
		assert_eq!(settings.get_enabled(), BitFlags::EMPTY);
	}

	#[test]
	fn schema_settings_set_from_all_enabled_check() {
		let settings = SchemaSettings::from(BitFlags::ALL);
		assert!(settings.is_enabled(SchemaSetting::AppendOnly));
		assert!(settings.is_enabled(SchemaSetting::SignatureRequired));
	}
}
