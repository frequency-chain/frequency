use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

use crate::parquet::base::ParquetBaseType;

use crate::parquet::string::ParquetStringType;
use crate::parquet::numeric::ParquetNumericType;
use crate::parquet::temporal::ParquetTemporalType;

/// Encapsulates label types for Parquet
#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParquetType {
  /// Base types (bool, int, etc)
  BaseType(ParquetBaseType),
  /// Strings
  StringType(ParquetStringType),
  /// Numbers
  NumericType(ParquetNumericType),
  /// Time
  TemporalType(ParquetTemporalType),
}

impl Default for ParquetType {
	fn default() -> Self {
		Self::BaseType(ParquetBaseType::Boolean)
	}
}
