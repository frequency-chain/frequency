use crate::{Config, Pallet, SchemaDescriptor, SchemaProtocolName, SchemaVersionId};
use common_primitives::schema::{IntentSettings, ModelType, PayloadLocation, SchemaId};
use frame_support::{pallet_prelude::*, storage_alias};
use sp_runtime::codec::{Decode, Encode};

/// Storage for old schema info struct data
/// - Key: Schema Id
/// - Value: [`SchemaInfo`](SchemaInfo)
#[storage_alias]
pub type SchemaInfos<T: Config> =
	StorageMap<Pallet<T>, Twox64Concat, SchemaId, SchemaInfo, OptionQuery>;

/// Storage for old schema name registry
/// - Key1: [`SchemaProtocolName`]
/// - Key2: [`SchemaDescriptor`]
/// - Value: [`SchemaVersionId`]
#[storage_alias]
pub type SchemaNameToIds<T: Config> = StorageDoubleMap<
	Pallet<T>,
	Blake2_128Concat,
	SchemaProtocolName,
	Blake2_128Concat,
	SchemaDescriptor,
	SchemaVersionId,
	ValueQuery,
>;

/// A structure defining Schema information (excluding the payload)
#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen)]
pub struct SchemaInfo {
	/// The type of model (AvroBinary, Parquet, etc.)
	pub model_type: ModelType,
	/// The payload location
	pub payload_location: PayloadLocation,
	/// additional control settings for the schema
	pub settings: IntentSettings,
	/// Defines if a schema has a name or not
	pub has_name: bool,
}
