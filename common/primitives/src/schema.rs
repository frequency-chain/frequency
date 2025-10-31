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

/// DelegationGrpup Id is the unique identifier for a DelegationGroup
pub type IntentGroupId = u16;

/// Intent Id is the unique identifier for an Intent
pub type IntentId = u16;

/// Schema Id is the unique identifier for a Schema
pub type SchemaId = u16;

/// Schema version number
// TODO: Remove once all traces of this are gone (ie, removed from runtime API)
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
	/// Intent setting to enforce append-only behavior on payload.
	/// Applied to Intents of type `PayloadLocation::Itemized`.
	AppendOnly,
	/// Intent may enforce signature requirement on payload.
	/// Applied to Intents of type `PayloadLocation::Itemized` or `PayloadLocation::Paginated`.
	SignatureRequired,
}

/// Wrapper type for `BitFlags<IntentSetting>` that implements `Codec`.
#[derive(Clone, Copy, PartialEq, Eq, Default, RuntimeDebug)]
pub struct SchemaSettings(pub BitFlags<IntentSetting>);

/// TODO: temporary alias until we can remove this type from the public API
pub type IntentSetting = SchemaSetting;

/// TODO: temporary alias until we can remove this type from the public API
pub type IntentSettings = SchemaSettings;

/// Status of a Schema
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
pub enum SchemaStatus {
	/// Schema is current and approved for writing
	Active,
	/// Schema is viable for writing, but is deprecated and may become unsupported soon
	Deprecated,
	/// Schema is unsupported; writing to this schema is prohibited
	Unsupported,
}

/// RPC response structure for an IntentGroup
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct IntentGroupResponse {
	/// The unique identifier for this IntentGroup
	pub intent_group_id: u16,
	/// The list of currently supported IntentIds for this IntentGroup
	pub intent_ids: Vec<IntentId>,
}

/// RPC response structure for an Intent
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct IntentResponse {
	/// The unique identifier for this Intent
	pub intent_id: IntentId,
	/// The payload location
	pub payload_location: PayloadLocation,
	/// settings for the Intent
	pub settings: Vec<IntentSetting>,
	/// The list of currently-supported SchemaIds for this Intent
	pub schema_ids: Option<Vec<SchemaId>>,
}

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
	/// The payload location associated with this Schema's associated Intent
	pub payload_location: PayloadLocation,
	/// The settings for this Schema's associated Intent
	pub settings: Vec<SchemaSetting>,
}

/// RPC Response form for a Schema V2
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct SchemaResponseV2 {
	/// The unique identifier for this Schema
	pub schema_id: SchemaId,
	/// The IntentId of the Intent that this Schema implements
	pub intent_id: IntentId,
	/// The data that represents how this schema is structured
	#[cfg_attr(feature = "std", serde(with = "as_string"))]
	pub model: Vec<u8>,
	/// The model format type for how the schema model is represented
	pub model_type: ModelType,
	/// The status of this Schema
	pub status: SchemaStatus,
	/// The payload location associated with this Schema's associated Intent
	pub payload_location: PayloadLocation,
	/// The settings for this Schema's associated Intent
	pub settings: Vec<IntentSetting>,
}

impl Into<SchemaResponse> for SchemaResponseV2 {
	/// Convert SchemaResponseV2 into SchemaResponse
	fn into(self) -> SchemaResponse {
		SchemaResponse {
			schema_id: self.schema_id,
			model: self.model,
			model_type: self.model_type,
			payload_location: self.payload_location,
			settings: self.settings,
		}
	}
}

/// RPC Response form for a Schema Info
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct SchemaInfoResponse {
	/// The unique identifier for this Schema
	pub schema_id: SchemaId,
	/// The IntentId of the Intent that this Schema implements
	pub intent_id: IntentId,
	/// The model format type for how the schema model is represented
	pub model_type: ModelType,
	/// The status of this Schema
	pub status: SchemaStatus,
	/// The payload location associated with this Schema's associated Intent
	pub payload_location: PayloadLocation,
	/// The settings for this Schema's associated Intent
	pub settings: Vec<IntentSetting>,
}

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
/// Enum type for the different types of entities that a FullyQualifiedName can represent
pub enum MappedEntityIdentifier {
	/// An Intent
	Intent(IntentId),
	/// An IntentGroup
	IntentGroup(IntentGroupId),
}

impl Default for MappedEntityIdentifier {
	fn default() -> Self {
		Self::Intent(Default::default())
	}
}

/// RPC response form for a name resolution lookup
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct NameLookupResponse {
	/// The name for this entity
	pub name: Vec<u8>,
	/// The resolved entity
	pub entity_id: MappedEntityIdentifier,
}

/// This allows other pallets to resolve Schema information. With generic SchemaId
pub trait SchemaProvider<SchemaId> {
	/// Gets the Schema details associated with this `SchemaId` if any
	fn get_schema_by_id(schema_id: SchemaId) -> Option<SchemaResponseV2>;

	/// Gets the Schema Info associated with this `SchemaId` if any
	fn get_schema_info_by_id(schema_id: SchemaId) -> Option<SchemaInfoResponse>;

	/// Gets the Intent associated with this `IntentId`, if any
	fn get_intent_by_id(intent_id: IntentId) -> Option<IntentResponse>;
}

/// This allows other Pallets to check the validity of schema ids.
pub trait SchemaValidator<SchemaId> {
	/// Checks that a collection of SchemaIds is all valid
	fn are_all_schema_ids_valid(schema_ids: &[SchemaId]) -> bool;

	/// Checks that all IntentIds in a collection are valid.
	fn are_all_intent_ids_valid(intent_ids: &[IntentId]) -> bool;

	/// Set the schema counter for testing purposes.
	#[cfg(any(feature = "std", feature = "runtime-benchmarks", test))]
	fn set_schema_count(n: SchemaId);

	/// Set the Intent counter for testing purposes.
	#[cfg(any(feature = "std", feature = "runtime-benchmarks", test))]
	fn set_intent_count(n: IntentId);
}

impl IntentSettings {
	/// Returns new SchemaSettings with all settings disabled
	pub fn all_disabled() -> Self {
		Self(BitFlags::EMPTY)
	}
	/// Get all setting enabled
	pub fn get_enabled(&self) -> BitFlags<IntentSetting> {
		self.0
	}
	/// Check if a setting is enabled
	pub fn is_enabled(&self, grant: IntentSetting) -> bool {
		self.0.contains(grant)
	}
	/// Enable a setting
	pub fn set(&mut self, grant: IntentSetting) {
		self.0.insert(grant)
	}
	/// Copy the settings from a BitFlags
	pub fn from(settings: BitFlags<IntentSetting>) -> Self {
		Self(settings)
	}
}
impl_codec_bitflags!(IntentSettings, u16, IntentSetting);

impl From<Vec<IntentSetting>> for IntentSettings {
	/// Copy the settings from a vector of individual settings enums
	fn from(settings: Vec<IntentSetting>) -> Self {
		Self(BitFlags::from_iter(settings))
	}
}

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
		let settings = IntentSettings::all_disabled();
		assert_eq!(settings.get_enabled(), BitFlags::EMPTY);
	}

	#[test]
	fn schema_settings_set_from_all_enabled_check() {
		let settings = IntentSettings::from(BitFlags::ALL);
		assert!(settings.is_enabled(IntentSetting::AppendOnly));
		assert!(settings.is_enabled(IntentSetting::SignatureRequired));
	}
}
