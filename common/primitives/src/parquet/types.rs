use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

use crate::parquet::base::ParquetBaseType;

use crate::parquet::string::ParquetStringType;
use crate::parquet::numeric::ParquetNumericType;
use crate::parquet::temporal::ParquetTemporalType;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen)]
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
