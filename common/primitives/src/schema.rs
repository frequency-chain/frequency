#[cfg(feature = "std")]
use crate::utils;
use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use utils::*;

pub type SchemaId = u16;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq)]
pub struct SchemaResponse {
	pub schema_id: SchemaId,
	#[cfg_attr(feature = "std", serde(with = "as_string"))]
	pub data: Vec<u8>,
}
