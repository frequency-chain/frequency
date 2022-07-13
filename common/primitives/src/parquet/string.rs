use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

/// Parquet String types: https://github.com/apache/parquet-format/blob/master/LogicalTypes.md
/// NOTE: We are not supporting ENUMs yet.

#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen, Serialize, Deserialize)]
pub enum ParquetStringType {
	/// Parquet strings
  String,
	/// Parquet UUIDs
  UUID
}

impl Default for ParquetStringType {
	fn default() -> Self {
		Self::String
	}
}
