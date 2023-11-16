//! Types for the Schema Pallet
use crate::{Config, Error};
use common_primitives::schema::{ModelType, PayloadLocation, SchemaId, SchemaSettings};
use frame_support::{ensure, pallet_prelude::ConstU32, traits::StorageVersion, BoundedVec};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::DispatchError;
use sp_std::fmt::Debug;
extern crate alloc;
use alloc::string::{String, ToString};
use sp_std::vec::Vec;

/// Current storage version of the schemas pallet.
pub const SCHEMA_STORAGE_VERSION: StorageVersion = StorageVersion::new(2);

/// The maximum size of schema name including all parts
pub const SCHEMA_NAME_BYTES_MAX: u32 = 32; // Hard limit of 32 bytes
/// A schema name following following structure NAMESPACE.DESCRIPTOR
pub type SchemaNamePayload = BoundedVec<u8, ConstU32<SCHEMA_NAME_BYTES_MAX>>;
/// The minimum size of a namespace in schema
pub const NAMESPACE_MIN: usize = 3;
/// The maximum size of a namespace in schema
pub const NAMESPACE_MAX: u32 = 15;
/// The minimum size of a schema descriptor
pub const DESCRIPTOR_MIN: usize = 1;
/// The maximum size of a schema descriptor
pub const DESCRIPTOR_MAX: u32 = 16;
/// separator character
pub const SEPARATOR_CHAR: char = '.';
const AUTO_ASSIGNED_NAMESPACE: &str = "AUTO_ASSIGNED";

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

#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen)]
/// A structure defining name of a schema
pub struct SchemaName {
	/// namespace or domain of the schema
	pub namespace: BoundedVec<u8, ConstU32<NAMESPACE_MAX>>,
	/// name or descriptor of this schema
	pub descriptor: BoundedVec<u8, ConstU32<DESCRIPTOR_MAX>>,
}

impl SchemaName {
	/// parses and verifies the request and returns the SchemaName type if successful
	pub fn try_parse<T: Config>(payload: SchemaNamePayload) -> Result<SchemaName, DispatchError> {
		// check if all ascii
		let mut str = String::from_utf8(payload.into_inner())
			.map_err(|_| Error::<T>::InvalidSchemaNameEncoding)?;
		ensure!(str.is_ascii(), Error::<T>::InvalidSchemaNameEncoding);

		// to canonical form
		str = str.to_lowercase();

		// check if alpha-numeric or separator character
		ensure!(
			str.chars().all(|c| c.is_ascii_alphanumeric() || c == SEPARATOR_CHAR),
			Error::<T>::InvalidSchemaNameCharacters
		);

		// split to namespace and descriptor
		let chunks: Vec<_> = str.split(SEPARATOR_CHAR).collect();
		ensure!(chunks.len() == 2, Error::<T>::InvalidSchemaNameStructure);

		// check namespace
		let namespace = BoundedVec::try_from(chunks[0].as_bytes().to_vec())
			.map_err(|_| Error::<T>::InvalidSchemaNamespaceLength)?;
		ensure!(NAMESPACE_MIN <= namespace.len(), Error::<T>::InvalidSchemaNamespaceLength);

		// check descriptor
		let descriptor = BoundedVec::try_from(chunks[1].as_bytes().to_vec())
			.map_err(|_| Error::<T>::InvalidSchemaDescriptorLength)?;
		ensure!(DESCRIPTOR_MIN <= descriptor.len(), Error::<T>::InvalidSchemaDescriptorLength);

		Ok(SchemaName { namespace, descriptor })
	}

	/// creates an auto assigned name and version for older extrinsics
	pub fn create_old<T: Config>(current_id: SchemaId) -> Result<SchemaName, DispatchError> {
		let namespace = BoundedVec::try_from(AUTO_ASSIGNED_NAMESPACE.to_string().into_bytes())
			.map_err(|_| Error::<T>::InvalidSchemaNamespaceLength)?;
		let descriptor = BoundedVec::try_from(current_id.to_string().into_bytes())
			.map_err(|_| Error::<T>::InvalidSchemaDescriptorLength)?;
		Ok(SchemaName { namespace, descriptor })
	}
}
