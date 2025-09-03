//! Types for the Schema Pallet
use crate::{Config, Error};
use common_primitives::schema::{
	IntentGroupId, IntentId, IntentSetting, IntentSettings, MappedEntityIdentifier, ModelType,
	NameLookupResponse, PayloadLocation, SchemaId, SchemaVersion, SchemaVersionResponse,
};
use core::fmt::Debug;
use frame_support::{ensure, pallet_prelude::ConstU32, traits::StorageVersion, BoundedVec};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::DispatchError;
extern crate alloc;
use alloc::{string::String, vec, vec::Vec};
use frame_support::traits::Len;

/// Current storage version of the schemas pallet.
pub const SCHEMA_STORAGE_VERSION: StorageVersion = StorageVersion::new(5);

/// The maximum size of a fully qualified name, including all parts and separators
pub const SCHEMA_NAME_BYTES_MAX: u32 = 32; // Hard limit of 32 bytes
/// A fully qualified name has the following structure PROTOCOL.DESCRIPTOR
pub type SchemaNamePayload = BoundedVec<u8, ConstU32<SCHEMA_NAME_BYTES_MAX>>;
/// schema namespace type
pub type SchemaProtocolName = BoundedVec<u8, ConstU32<PROTOCOL_NAME_MAX>>;
/// schema descriptor type
pub type SchemaDescriptor = BoundedVec<u8, ConstU32<DESCRIPTOR_MAX>>;
/// The minimum size of a namespace in schema
pub const PROTOCOL_NAME_MIN: u32 = 3;
/// The maximum size of a namespace in schema
pub const PROTOCOL_NAME_MAX: u32 = SCHEMA_NAME_BYTES_MAX - (DESCRIPTOR_MIN + 1);
/// The minimum size of a schema descriptor
pub const DESCRIPTOR_MIN: u32 = 1;
/// The maximum size of a schema descriptor
pub const DESCRIPTOR_MAX: u32 = SCHEMA_NAME_BYTES_MAX - (PROTOCOL_NAME_MIN + 1);
/// Maximum number of intents allowed per delegation group
pub const MAX_INTENTS_PER_DELEGATION_GROUP: u32 = 10;
/// separator character
pub const SEPARATOR_CHAR: char = '.';
/// maximum number of versions for a certain schema name
/// -1 is to avoid overflow when converting the (index + 1) to `SchemaVersion` in `SchemaVersionId`
pub const MAX_NUMBER_OF_VERSIONS: u32 = SchemaVersion::MAX as u32 - 1;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
/// Genesis Schemas need a way to load up and this is it!
pub struct GenesisSchema {
	/// The SchemaId
	pub schema_id: SchemaId,
	/// The IntentId of the Intent that this Schema implements
	pub intent_id: IntentId,
	/// The type of model (AvroBinary, Parquet, etc.)
	pub model_type: ModelType,
	/// The Payload Model
	pub model: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
/// Genesis Intents need a way to load up and this is it!
pub struct GenesisIntent {
	/// The IntentId
	pub intent_id: IntentId,
	/// The payload location
	pub payload_location: PayloadLocation,
	/// Schema Full Name: {Protocol}.{Descriptor}
	pub name: String,
	/// Settings
	pub settings: Vec<IntentSetting>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
/// Genesis IntentGroups need a way to load up and this is it!
pub struct GenesisIntentGroup {
	/// The IntentGroupId
	pub intent_group_id: IntentGroupId,
	/// The name: {Protocol}.{Descriptor}
	pub name: String,
	/// The list of Intents in the group
	pub intent_ids: Vec<IntentId>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
/// Overall Genesis config loading structure for the entire pallet
pub struct GenesisSchemasPalletConfig {
	/// The list of Schemas
	pub schemas: Vec<GenesisSchema>,
	/// The list of Intents
	pub intents: Vec<GenesisIntent>,
	/// The list of IntentGroups
	pub intent_groups: Vec<GenesisIntentGroup>,
}

#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
/// A structure defining an IntentGroup
pub struct IntentGroup<T: Config> {
	/// The list of Intents in the DelegationGroup
	pub intent_ids: BoundedVec<IntentId, T::MaxIntentsPerIntentGroup>,
}

#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen)]
/// A structure defining Intent information
pub struct IntentInfo {
	/// The payload location
	pub payload_location: PayloadLocation,
	/// additional control settings for the schema
	pub settings: IntentSettings,
}

#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen)]
/// A structure defining Schema information (excluding the payload)
pub struct SchemaInfo {
	/// The IntentId of the Intent that this Schema implements
	pub intent_id: IntentId,
	/// The type of model (AvroBinary, Parquet, etc.)
	pub model_type: ModelType,
	/// The payload location (inherited from the Intent)
	pub payload_location: PayloadLocation,
	/// additional control settings (inherited from the Intent)
	pub settings: IntentSettings,
}

#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen)]
/// A structure defining name of a schema
pub struct SchemaName {
	/// namespace or domain of the schema
	pub namespace: SchemaProtocolName,
	/// name or descriptor of this schema
	pub descriptor: SchemaDescriptor,
}

/// A structure defining a fully qualified name of an entity
// TODO: type alias for now, until we finish converting the Schemas and deprecate the old methods
pub type FullyQualifiedName = SchemaName;

#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen, Default)]
/// A structure defining name of a schema
pub struct SchemaVersionId {
	/// the index of each item + 1 is considered as their version.
	/// Ex: the schemaId located in `ids[2]` is for version number 3
	pub ids: BoundedVec<SchemaId, ConstU32<MAX_NUMBER_OF_VERSIONS>>,
}

impl SchemaName {
	/// parses and verifies the request and returns the SchemaName type if successful
	///
	/// Note: passing `require_descriptor: false` is intended for RPC methods that search the
	/// name registry by protocol. Operations that validate name creation should always pass
	/// `require_descriptor: true`.
	///
	/// # Errors
	/// * [`Error::InvalidSchemaNameEncoding`] - The name has an invalid encoding.
	/// * [`Error::InvalidSchemaNameCharacters`] - The name contains invalid characters.
	/// * [`Error::InvalidSchemaNameStructure`] - The name has an invalid structure (i.e., not `protocol.descriptor`).
	/// * [`Error::InvalidSchemaNameLength`] - The name exceeds the allowed overall name length.
	/// * [`Error::InvalidSchemaNamespaceLength`] - The protocol portion of the name exceeds the max allowed length.
	/// * [`Error::InvalidSchemaDescriptorLength`] - The descriptor portion of the name exceeds the max allowed length.
	pub fn try_parse<T: Config>(
		payload: SchemaNamePayload,
		require_descriptor: bool,
	) -> Result<SchemaName, DispatchError> {
		// check if all ascii
		let mut str = String::from_utf8(payload.into_inner())
			.map_err(|_| Error::<T>::InvalidSchemaNameEncoding)?;
		ensure!(str.is_ascii(), Error::<T>::InvalidSchemaNameEncoding);

		// to canonical form
		str = String::from(str.to_lowercase().trim());

		// only allow the following:
		// - alphanumeric characters
		// - '-' (hyphen)
		// - SEPARATOR_CHAR (period)
		ensure!(
			str.chars()
				.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == SEPARATOR_CHAR),
			Error::<T>::InvalidSchemaNameCharacters
		);

		// split to namespace and descriptor
		let chunks: Vec<_> = str.split(SEPARATOR_CHAR).collect();
		ensure!(
			chunks.len() == 2 || (chunks.len() == 1 && !require_descriptor),
			Error::<T>::InvalidSchemaNameStructure
		);

		// check namespace
		let namespace = BoundedVec::try_from(chunks[0].as_bytes().to_vec())
			.map_err(|_| Error::<T>::InvalidSchemaNamespaceLength)?;
		ensure!(
			PROTOCOL_NAME_MIN <= namespace.len() as u32,
			Error::<T>::InvalidSchemaNamespaceLength
		);

		// check that it starts with an alphabetic character.
		// this ensures:
		// - does not start with '-'
		// - does not start with a number
		// - does not start with a SEPARATOR_CHAR
		// (This also handles the case where the value is all numeric, because it would also
		// start with a decimal digit.)
		ensure!(namespace[0].is_ascii_alphabetic(), Error::<T>::InvalidSchemaNameCharacters);
		// should not end with -
		ensure!(!namespace.ends_with(b"-"), Error::<T>::InvalidSchemaNameStructure);

		// check descriptor
		let descriptor = match chunks.len() == 2 {
			true => {
				let descriptor = BoundedVec::try_from(chunks[1].as_bytes().to_vec())
					.map_err(|_| Error::<T>::InvalidSchemaDescriptorLength)?;
				ensure!(
					DESCRIPTOR_MIN <= descriptor.len() as u32,
					Error::<T>::InvalidSchemaDescriptorLength
				);
				// check that it starts with an alphabetic character.
				// this ensures:
				// - does not start with '-'
				// - does not start with a number
				// - does not start with a SEPARATOR_CHAR
				// (This also handles the case where the value is all numeric, because it would also
				// start with a decimal digit.)
				ensure!(
					descriptor[0].is_ascii_alphabetic(),
					Error::<T>::InvalidSchemaNameCharacters
				);
				// should end with -
				ensure!(!descriptor.ends_with(b"-"), Error::<T>::InvalidSchemaNameStructure);
				descriptor
			},
			false => BoundedVec::default(),
		};

		Ok(SchemaName { namespace, descriptor })
	}

	/// get the combined name namespace.descriptor
	pub fn get_combined_name(&self) -> Vec<u8> {
		[
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
		let is_new = !self.ids.iter().any(|id| id == &schema_id);
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

/// trait to create a response from a name and entity identifier
pub trait ConvertToResponse<I, R> {
	/// method to convert to response
	fn convert_to_response(&self, name: &I) -> R;
}

impl ConvertToResponse<SchemaName, NameLookupResponse> for MappedEntityIdentifier {
	fn convert_to_response(&self, name: &SchemaName) -> NameLookupResponse {
		NameLookupResponse { name: name.get_combined_name(), entity_id: *self }
	}
}
