use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen, Serialize, Deserialize)]
pub enum ParquetBaseType {
  Boolean,
  Int32,
  Int64,
  Float,
  Double,
  ByteArray,
  FixedLengthByteArray
}

impl Default for ParquetBaseType {
	fn default() -> Self {
		Self::Boolean
	}
}
