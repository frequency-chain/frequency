use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

/// Parquet temporal types: https://github.com/apache/parquet-format/blob/master/LogicalTypes.md
#[derive(Clone, PartialEq, Debug, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ParquetTemporalType {
	/// Parquet dates
	Date,
	/// Parquet intervals
	Interval,
	/// Time
	Time(ParquetTime),
	/// Timestamps
	Timestamp(ParquetTimestamp),
}

/// Parquet time: https://github.com/apache/parquet-format/blob/master/LogicalTypes.md
#[derive(Clone, PartialEq, Debug, Eq, Serialize, Deserialize)]
pub struct ParquetTime {
	is_adjusted_to_utc: bool,
	unit: ParquetTimeUnit,
}

/// Parquet timestamps: https://github.com/apache/parquet-format/blob/master/LogicalTypes.md
#[derive(Clone, PartialEq, Debug, Eq, Serialize, Deserialize)]
pub struct ParquetTimestamp {
	is_adjusted_to_utc: bool,
	unit: ParquetTimeUnit,
}

/// Units of time
#[derive(Clone, PartialEq, Debug, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
enum ParquetTimeUnit {
	/// milliseconds
	Millis,
	/// microseconds
	Micros,
	/// nanoseconds
	Nanos,
}
