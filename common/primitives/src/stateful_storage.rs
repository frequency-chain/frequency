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

/// A type for responding with an single Message in an RPC-call dependent on schema model
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct PageResponse {
	///  index or number
	pub index_number: u16,
	///  Message source account id (the original source).
	pub msa_id: MessageSourceId,
	///  Schema id of the
	pub schema_id: SchemaId,
	/// Serialized data in a the schemas.
	#[cfg_attr(feature = "std", serde(with = "as_hex", default))]
	pub payload: Vec<u8>,
}

impl PageResponse {
	/// Returns a new instance with associated parameters
	pub fn new(
		index_number: u16,
		msa_id: MessageSourceId,
		schema_id: SchemaId,
		payload: Vec<u8>,
	) -> Self {
		PageResponse { index_number, msa_id, schema_id, payload }
	}
}
