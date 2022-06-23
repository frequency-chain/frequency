//! Types for the Schema Pallet
use super::*;
use scale_info::TypeInfo;

use codec::{Decode, Encode, MaxEncodedLen};

use frame_support::traits::Get;

/// Types of modeling in which a message payload may be defined
#[derive(Default, Clone, Encode, Decode, Debug, TypeInfo, PartialEq, Eq, MaxEncodedLen)]
pub enum ModelType {
	/// Message payload modeled with Apache Avro
	#[default]
	AvroBinary,
}

#[derive(Default, Clone, Encode, Decode, PartialEq, TypeInfo, Debug, Eq, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
/// A structure defining a Schema
pub struct Schema<T: Get<u32>> {
	/// Model Type
	pub model_type: ModelType,
	/// Model
	pub model: BoundedVec<u8, T>,
}
// Constrain T to more than one trait? Such that it is also required to implement MaxEncodedLen?
//
