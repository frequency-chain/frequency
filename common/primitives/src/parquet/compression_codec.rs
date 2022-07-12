use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen, Serialize, Deserialize)]
pub enum CompressionCodec {
  Uncompressed
}

impl Default for CompressionCodec {
	fn default() -> Self {
		Self::Uncompressed
	}
}
