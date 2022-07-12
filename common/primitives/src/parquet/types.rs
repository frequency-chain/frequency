use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

use crate::parquet::base::ParquetBaseType;

use crate::parquet::string::ParquetStringType;
use crate::parquet::numeric::ParquetNumericType;
use crate::parquet::temporal::ParquetTemporalType;

#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen, Serialize, Deserialize)]
pub enum ParquetType {
  BaseType(ParquetBaseType),
  StringType(ParquetStringType),
  NumericType(ParquetNumericType),
  TemporalType(ParquetTemporalType),
}

impl Default for ParquetType {
	fn default() -> Self {
		Self::BaseType(ParquetBaseType::Boolean)
	}
}
