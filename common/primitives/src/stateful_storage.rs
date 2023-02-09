use crate::msa::{MessageSourceId, SchemaId};
#[cfg(feature = "std")]
use crate::utils;
use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use utils::*;

/// PageId is the unique identifier for a Page in Stateful Storage
pub type PageId = u16;

/// A type to expose different types of stateful storages
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct StatefulStorageResponse {
	///  index or id of this storage
	/// for Itemized storages this is going to be index of the item
	/// for Paginated storages this is going to be the Id of the page
	pub index_id: PageId,
	///  Message source account id (the original source).
	pub msa_id: MessageSourceId,
	///  Schema id of the
	pub schema_id: SchemaId,
	/// Serialized data in a the schemas.
	#[cfg_attr(feature = "std", serde(with = "as_hex", default))]
	pub payload: Vec<u8>,
}

impl StatefulStorageResponse {
	/// Returns a new instance with associated parameters
	pub fn new(
		index_number: u16,
		msa_id: MessageSourceId,
		schema_id: SchemaId,
		payload: Vec<u8>,
	) -> Self {
		StatefulStorageResponse { index_id: index_number, msa_id, schema_id, payload }
	}
}
