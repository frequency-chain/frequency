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
/// PageHash is the type/size of hash of the page content.
pub type PageHash = u32;

/// A type to expose paginated type of stateful storages
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct PaginatedStorageResponse {
	///  id of the page
	pub page_id: PageId,
	///  Message source account id (the original source).
	pub msa_id: MessageSourceId,
	///  Schema id of the
	pub schema_id: SchemaId,
	/// Hash of the page content
	pub content_hash: PageHash,
	/// Serialized data in a the schemas.
	#[cfg_attr(feature = "std", serde(with = "as_hex", default))]
	pub payload: Vec<u8>,
}

/// A type to expose itemized page of stateful storages
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct ItemizedStoragePageResponse {
	///  Message source account id (the original source).
	pub msa_id: MessageSourceId,
	///  Schema id of the
	pub schema_id: SchemaId,
	/// Hash of the page content
	pub content_hash: PageHash,
	/// Items in a page
	pub items: Vec<ItemizedStorageResponse>,
}

/// A type to expose itemized type of stateful storages
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct ItemizedStorageResponse {
	///  index of item
	pub index: u16,
	/// Serialized data of item.
	#[cfg_attr(feature = "std", serde(with = "as_hex", default))]
	pub payload: Vec<u8>,
}

impl PaginatedStorageResponse {
	/// Returns a new instance with associated parameters
	pub fn new(
		index_number: u16,
		msa_id: MessageSourceId,
		schema_id: SchemaId,
		payload: Vec<u8>,
		content_hash: PageHash,
	) -> Self {
		PaginatedStorageResponse { page_id: index_number, msa_id, schema_id, payload, content_hash }
	}
}

impl ItemizedStorageResponse {
	/// Returns a new instance with associated parameters
	pub fn new(index: u16, payload: Vec<u8>) -> Self {
		ItemizedStorageResponse { index, payload }
	}
}

impl ItemizedStoragePageResponse {
	/// Returns a new instance with associated parameters
	pub fn new(
		msa_id: MessageSourceId,
		schema_id: SchemaId,
		content_hash: PageHash,
		items: Vec<ItemizedStorageResponse>,
	) -> Self {
		ItemizedStoragePageResponse { msa_id, schema_id, content_hash, items }
	}
}
