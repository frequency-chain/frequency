use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

/// Base types
#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen, Serialize, Deserialize)]
pub enum ParquetBaseType {
  /// Encapsulates true / false values
  Boolean,
  /// Encapsulates 32 bit ints
  Int32,
  /// Encapsulates 64 bit ints
  Int64,
  /// Encapsulates floats
  Float,
  /// Encapsulates doubles
  Double,
  /// Encapsulates arrays
  ByteArray,
  /// Encapsulates fixed length arrays
  FixedLengthByteArray
}

impl Default for ParquetBaseType {
	fn default() -> Self {
		Self::Boolean
	}
}
