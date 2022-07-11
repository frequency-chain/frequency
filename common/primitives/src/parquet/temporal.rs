use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen)]
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


#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen)]
pub struct ParquetTime {
  is_adjusted_to_utc: bool,
  unit: ParquetTimeUnit
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen)]
pub struct ParquetTimestamp {
  is_adjusted_to_utc: bool,
  unit: ParquetTimeUnit
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen)]
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
