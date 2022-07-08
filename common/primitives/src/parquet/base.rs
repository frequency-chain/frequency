use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen)]
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
