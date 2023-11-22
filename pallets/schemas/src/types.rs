//! Types for the Schema Pallet
use crate::{Config, Error};
use common_primitives::schema::{
	ModelType, PayloadLocation, SchemaId, SchemaSettings, SchemaVersion, SchemaVersionResponse,
};
use frame_support::{ensure, pallet_prelude::ConstU32, traits::StorageVersion, BoundedVec};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::DispatchError;
use sp_std::fmt::Debug;
extern crate alloc;
use alloc::string::String;
use frame_support::traits::Len;
use sp_std::{vec, vec::*};

/// Current storage version of the schemas pallet.
pub const SCHEMA_STORAGE_VERSION: StorageVersion = StorageVersion::new(3);

/// The maximum size of schema name including all parts
pub const SCHEMA_NAME_BYTES_MAX: u32 = 32; // Hard limit of 32 bytes
/// A schema name following following structure NAMESPACE.DESCRIPTOR
pub type SchemaNamePayload = BoundedVec<u8, ConstU32<SCHEMA_NAME_BYTES_MAX>>;
/// schema namespace type
pub type SchemaNamespace = BoundedVec<u8, ConstU32<NAMESPACE_MAX>>;
/// schema descriptor type
pub type SchemaDescriptor = BoundedVec<u8, ConstU32<DESCRIPTOR_MAX>>;
/// The minimum size of a namespace in schema
pub const NAMESPACE_MIN: u32 = 3;
/// The maximum size of a namespace in schema
pub const NAMESPACE_MAX: u32 = SCHEMA_NAME_BYTES_MAX - (DESCRIPTOR_MIN + 1);
/// The minimum size of a schema descriptor
pub const DESCRIPTOR_MIN: u32 = 1;
/// The maximum size of a schema descriptor
pub const DESCRIPTOR_MAX: u32 = SCHEMA_NAME_BYTES_MAX - (NAMESPACE_MIN + 1);
/// separator character
pub const SEPARATOR_CHAR: char = '.';
/// maximum number of versions for a certain schema name
/// -1 is to avoid overflow when converting the (index + 1) to `SchemaVersion` in `SchemaVersionId`
pub const MAX_NUMBER_OF_VERSIONS: u32 = SchemaVersion::MAX as u32 - 1;

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
	pub namespace: SchemaNamespace,
	/// name or descriptor of this schema
	pub descriptor: SchemaDescriptor,
}

#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen, Default)]
/// A structure defining name of a schema
pub struct SchemaVersionId {
	/// the index of each item + 1 is considered as their version.
	/// Ex: the schemaId located in `ids[2]` is for version number 3
	pub ids: BoundedVec<SchemaId, ConstU32<MAX_NUMBER_OF_VERSIONS>>,
}

impl SchemaName {
	/// parses and verifies the request and returns the SchemaName type if successful
	pub fn try_parse<T: Config>(
		payload: SchemaNamePayload,
		is_strict: bool,
	) -> Result<SchemaName, DispatchError> {
		// check if all ascii
		let mut str = String::from_utf8(payload.into_inner())
			.map_err(|_| Error::<T>::InvalidSchemaNameEncoding)?;
		ensure!(str.is_ascii(), Error::<T>::InvalidSchemaNameEncoding);

		// to canonical form
		str = str.to_lowercase();

		// check if alphabetic or - or separator character
		ensure!(
			str.chars().all(|c| c.is_ascii_alphabetic() || c == '-' || c == SEPARATOR_CHAR),
			Error::<T>::InvalidSchemaNameCharacters
		);

		// split to namespace and descriptor
		let chunks: Vec<_> = str.split(SEPARATOR_CHAR).collect();
		ensure!(
			chunks.len() == 2 || (chunks.len() == 1 && is_strict == false),
			Error::<T>::InvalidSchemaNameStructure
		);

		// check namespace
		let namespace = BoundedVec::try_from(chunks[0].as_bytes().to_vec())
			.map_err(|_| Error::<T>::InvalidSchemaNamespaceLength)?;
		ensure!(NAMESPACE_MIN <= namespace.len() as u32, Error::<T>::InvalidSchemaNamespaceLength);

		// check descriptor
		let descriptor = match chunks.len() == 2 {
			true => {
				let descriptor = BoundedVec::try_from(chunks[1].as_bytes().to_vec())
					.map_err(|_| Error::<T>::InvalidSchemaDescriptorLength)?;
				ensure!(
					DESCRIPTOR_MIN <= descriptor.len() as u32,
					Error::<T>::InvalidSchemaDescriptorLength
				);
				descriptor
			},
			false => BoundedVec::default(),
		};

		Ok(SchemaName { namespace, descriptor })
	}

	/// get the combined name namespace.descriptor
	pub fn get_combined_name(&self) -> Vec<u8> {
		vec![
			self.namespace.clone().into_inner(),
			vec![SEPARATOR_CHAR as u8],
			self.descriptor.clone().into_inner(),
		]
		.concat()
	}

	/// creates a new SchemaName using provided descriptor
	pub fn new_with_descriptor(&self, descriptor: SchemaDescriptor) -> Self {
		Self { namespace: self.namespace.clone(), descriptor }
	}

	/// returns true if the descriptor exists
	pub fn descriptor_exists(&self) -> bool {
		self.descriptor.len() > 0
	}
}

impl SchemaVersionId {
	/// adds a new schema id and returns the version for that schema_id
	pub fn add<T: Config>(&mut self, schema_id: SchemaId) -> Result<SchemaVersion, DispatchError> {
		let is_new = self.ids.iter().find(|id| *id == &schema_id).is_none();
		ensure!(is_new, Error::<T>::SchemaIdAlreadyExists);
		self.ids
			.try_push(schema_id)
			.map_err(|_| Error::<T>::ExceedsMaxNumberOfVersions)?;
		let version = self.ids.len() as SchemaVersion;
		Ok(version)
	}

	/// convert into a response vector
	pub fn convert_to_response(&self, schema_name: &SchemaName) -> Vec<SchemaVersionResponse> {
		self.ids
			.iter()
			.enumerate()
			.map(|(index, schema_id)| SchemaVersionResponse {
				schema_name: schema_name.get_combined_name(),
				schema_id: *schema_id,
				schema_version: (index + 1) as SchemaVersion,
			})
			.collect()
	}
}
