//! Types for the Schema Pallet
use frame_support::{traits::Get, BoundedVec};
use scale_info::TypeInfo;

use codec::{Decode, Encode, MaxEncodedLen};

/// Types of modeling in which a message payload may be defined
#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen)]
pub enum ModelType {
	/// Message payload modeled with Apache Avro
	#[default]
	AvroBinary,
}

#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxModelSize))]
/// A structure defining a Schema
pub struct Schema<MaxModelSize>
where
	MaxModelSize: Get<u32>,
{
	/// The type of model (AvroBinary, Parquery, etc.)
	pub model_type: ModelType,
	/// Defines the structure of the message payload using model_type
	pub model: BoundedVec<u8, MaxModelSize>,
}
