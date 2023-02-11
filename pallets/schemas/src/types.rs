//! Types for the Schema Pallet
use codec::{Decode, Encode, EncodeLike, MaxEncodedLen};
use common_primitives::{
	impl_codec_bitflags,
	schema::{ModelType, PayloadLocation},
};
use enumflags2::{bitflags, BitFlags};
use frame_support::{traits::Get, BoundedVec, RuntimeDebug};
use scale_info::{build::Fields, meta_type, Path, Type, TypeInfo, TypeParameter};
use sp_std::vec;

#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen)]
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
	/// grants for the schema
	pub grants: Grants,
}

/// Support for up to 16 user-enabled features on a collection.
#[bitflags]
#[repr(u16)]
#[derive(Copy, Clone, RuntimeDebug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum Grant {
	/// Items in this collection are transferable.
	AppendOnly,
	/// The metadata of this collection can be modified.
	SignatureRequired,
}

/// Wrapper type for `BitFlags<Grant>` that implements `Codec`.
#[derive(Clone, Copy, PartialEq, Eq, Default, RuntimeDebug)]
pub struct Grants(pub BitFlags<Grant>);

impl Grants {
	/// some docs
	pub fn all_disabled() -> Self {
		Self(BitFlags::EMPTY)
	}
	/// some docs
	pub fn get_enabled(&self) -> BitFlags<Grant> {
		self.0
	}
	/// some docs
	pub fn is_enabled(&self, setting: Grant) -> bool {
		self.0.contains(setting)
	}
	/// some docs
	pub fn set(&mut self, setting: Grant) {
		self.0.insert(setting)
	}
	/// some docs
	pub fn from(settings: BitFlags<Grant>) -> Self {
		Self(settings)
	}
}
impl_codec_bitflags!(Grants, u16, Grant);
