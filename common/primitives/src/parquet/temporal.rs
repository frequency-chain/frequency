use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen, Serialize, Deserialize)]
pub enum ParquetTemporalType {
  Date,
  Interval,
  Time(ParquetTime),
  Timestamp(ParquetTimestamp),  
}

impl Default for ParquetTemporalType {
	fn default() -> Self {
		Self::Timestamp(ParquetTimestamp::default())
	}
}

#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen, Serialize, Deserialize)]
pub struct ParquetTime {
  is_adjusted_to_utc: bool,
  unit: ParquetTimeUnit
}

#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen, Serialize, Deserialize)]
pub struct ParquetTimestamp {
  is_adjusted_to_utc: bool,
  unit: ParquetTimeUnit
}

#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen, Serialize, Deserialize)]
enum ParquetTimeUnit {
  MILLIS,
  MICROS,
  NANOS
}

impl Default for ParquetTimeUnit {
	fn default() -> Self {
		Self::MILLIS
	}
}
