use serde::{Deserialize, Serialize};

use crate::parquet::base::ParquetBaseType;

use crate::parquet::{
	numeric::ParquetNumericType, string::ParquetStringType, temporal::ParquetTemporalType,
};

/// Encapsulates label types for Parquet
#[derive(Clone, PartialEq, Debug, Eq, Serialize, Deserialize)]
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
