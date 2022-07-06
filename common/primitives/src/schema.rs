#[cfg(feature = "std")]
use crate::utils;
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use utils::*;

/// Schema Id is the unique identifier for a Schema
pub type SchemaId = u16;

/// Types of modeling in which a message payload may be defined
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen)]
pub enum ModelType {
	/// Message payload modeled with Apache Avro: https://avro.apache.org/docs/current/spec.html
	AvroBinary,
}

impl Default for ModelType {
	fn default() -> Self {
		Self::AvroBinary
	}
}

/// RPC Response form for a Schema
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct SchemaResponse {
	/// The unique identifier for this Schema
	pub schema_id: SchemaId,
	/// The data that represents how this schema is structured
	#[cfg_attr(feature = "std", serde(with = "as_string"))]
	pub model: Vec<u8>,
	/// The model format type for how the schema model is represented
	pub model_type: ModelType,
}

/// This allows other pallets to resolve Schema information.
pub trait SchemaProvider {
	/// Type used to associate a key to a Schema.
	type SchemaId;

	/// Gets the Schema Id associated with this `SchemaId` if any
	/// # Arguments
	/// * `schema_id` - The `SchemaId` to lookup
	/// # Returns
	/// * `Option<SchemaResponse>`
	/// # Remarks
	/// This function is used to resolve a Schema from a SchemaId.
	fn get_schema_by_id(schema_id: Self::SchemaId) -> Option<SchemaResponse>;
}
