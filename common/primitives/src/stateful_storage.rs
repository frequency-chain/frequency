use crate::msa::MessageSourceId;
#[cfg(feature = "std")]
use crate::utils;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
extern crate alloc;
use crate::schema::{IntentId, SchemaId};
use alloc::vec::Vec;
#[cfg(feature = "std")]
use utils::*;

/// PageId is the unique identifier for a Page in Stateful Storage
pub type PageId = u16;
/// PageHash is the type/size of the hashed page content.
pub type PageHash = u32;
/// PageNonce is the type/size of a nonce value embedded into a Page
pub type PageNonce = u16;

/// A type to expose paginated type of stateful storage
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct PaginatedStorageResponse {
	///  id of the page
	pub page_id: PageId,
	///  Message source account id (the original source).
	pub msa_id: MessageSourceId,
	///  Schema id of requested storage
	pub schema_id: SchemaId,
	/// Hash of the page content
	pub content_hash: PageHash,
	/// Nonce of the page
	pub nonce: PageNonce,
	/// Serialized data in the schema.
	#[cfg_attr(feature = "std", serde(with = "as_hex", default))]
	pub payload: Vec<u8>,
}

/// A type to expose itemized page of stateful storage
/// A type to expose paginated type of stateful storage
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct PaginatedStorageResponseV2 {
	/// IntentId of the page
	pub intent_id: IntentId,
	///  id of the page
	pub page_id: PageId,
	///  Message source account id (the original source).
	pub msa_id: MessageSourceId,
	///  Schema id of requested storage
	pub schema_id: SchemaId,
	/// Hash of the page content
	pub content_hash: PageHash,
	/// Nonce of the page
	pub nonce: PageNonce,
	/// Serialized data in the schema.
	#[cfg_attr(feature = "std", serde(with = "as_hex", default))]
	pub payload: Vec<u8>,
}

/// A type to expose itemized page of stateful storage
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct ItemizedStoragePageResponse {
	///  Message source account id (the original source).
	pub msa_id: MessageSourceId,
	///  Schema id of requested storage
	pub schema_id: SchemaId,
	/// Hash of the page content
	pub content_hash: PageHash,
	/// Nonce of the page
	pub nonce: PageNonce,
	/// Items in a page
	pub items: Vec<ItemizedStorageResponse>,
}

/// A type to expose itemized page of stateful storage
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct ItemizedStoragePageResponseV2 {
	///  Message source account id (the original source).
	pub msa_id: MessageSourceId,
	///  Intent id of requested storage
	pub intent_id: IntentId,
	/// Hash of the page content
	pub content_hash: PageHash,
	/// Nonce of the page
	pub nonce: PageNonce,
	/// Items in a page
	pub items: Vec<ItemizedStorageResponseV2>,
}

/// A type to expose itemized type of stateful storage
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct ItemizedStorageResponse {
	///  index of item
	pub index: u16,
	/// Serialized data of item.
	#[cfg_attr(feature = "std", serde(with = "as_hex", default))]
	pub payload: Vec<u8>,
}

/// A type to expose itemized type of stateful storage
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct ItemizedStorageResponseV2 {
	///  index of item
	pub index: u16,
	/// SchemaId of the item.
	pub schema_id: SchemaId,
	/// Serialized data of item.
	#[cfg_attr(feature = "std", serde(with = "as_hex", default))]
	pub payload: Vec<u8>,
}

impl PaginatedStorageResponseV2 {
	/// Returns a new instance with associated parameters
	pub fn new(
		index_number: u16,
		msa_id: MessageSourceId,
		intent_id: IntentId,
		schema_id: SchemaId,
		content_hash: PageHash,
		nonce: PageNonce,
		payload: Vec<u8>,
	) -> Self {
		PaginatedStorageResponseV2 {
			page_id: index_number,
			msa_id,
			intent_id,
			schema_id,
			payload,
			content_hash,
			nonce,
		}
	}
}

impl Into<PaginatedStorageResponse> for PaginatedStorageResponseV2 {
	fn into(self) -> PaginatedStorageResponse {
		PaginatedStorageResponse {
			page_id: self.page_id,
			msa_id: self.msa_id,
			schema_id: self.schema_id,
			payload: self.payload,
			content_hash: self.content_hash,
			nonce: self.nonce,
		}
	}
}

impl ItemizedStorageResponseV2 {
	/// Returns a new instance with associated parameters
	pub fn new(index: u16, schema_id: SchemaId, payload: Vec<u8>) -> Self {
		ItemizedStorageResponseV2 { index, schema_id, payload }
	}
}

impl Into<ItemizedStorageResponse> for ItemizedStorageResponseV2 {
	fn into(self) -> ItemizedStorageResponse {
		ItemizedStorageResponse { index: self.index, payload: self.payload }
	}
}

impl ItemizedStoragePageResponseV2 {
	/// Returns a new instance with associated parameters
	pub fn new(
		msa_id: MessageSourceId,
		intent_id: IntentId,
		content_hash: PageHash,
		nonce: PageNonce,
		items: Vec<ItemizedStorageResponseV2>,
	) -> Self {
		ItemizedStoragePageResponseV2 { msa_id, intent_id, content_hash, items, nonce }
	}
}

impl Into<ItemizedStoragePageResponse> for ItemizedStoragePageResponseV2 {
	fn into(self) -> ItemizedStoragePageResponse {
		ItemizedStoragePageResponse {
			msa_id: self.msa_id,
			schema_id: self.items.get(0).map(|item| item.schema_id).unwrap_or(0),
			content_hash: self.content_hash,
			nonce: self.nonce,
			items: self.items.into_iter().map(|item| item.into()).collect(),
		}
	}
}
