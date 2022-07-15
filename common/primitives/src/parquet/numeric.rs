use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

/// Parquet numeric types: https://github.com/apache/parquet-format/blob/master/LogicalTypes.md
#[derive(
	Clone, PartialEq, Debug, Eq, Serialize, Deserialize,
)]
pub enum ParquetNumericType {
	/// Integers
	Integer(ParquetInteger),
	/// Decimals
	Decimal(ParquetDecimal),
}

// impl Default for ParquetNumericType {
// 	fn default() -> Self {
// 		Self::Integer(ParquetInteger::default())
// 	}
// }

/// Parquet Integers
#[derive(
	Clone,
	PartialEq,
	Debug,
	Eq,
	Serialize,
	Deserialize,
)]
pub struct ParquetInteger {
	bit_width: u8,
	sign: bool,
}

/// Parquet Decimals
#[derive(
	Clone,
	PartialEq,
	Debug,
	Eq,
	Serialize,
	Deserialize,
)]
pub struct ParquetDecimal {
	scale: u8,

	// Note: in theory, precision is unbounded. But a u8 should be fine for practical cases
	precision: u8,
}
