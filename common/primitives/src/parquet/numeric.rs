use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

/// Parquet numeric types: https://github.com/apache/parquet-format/blob/master/LogicalTypes.md
#[derive(
	Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen, Serialize, Deserialize,
)]
pub enum ParquetNumericType {
	/// Integers
	Integer(ParquetInteger),
	/// Decimals
	Decimal(ParquetDecimal),
}

impl Default for ParquetNumericType {
	fn default() -> Self {
		Self::Integer(ParquetInteger::default())
	}
}

/// Parquet Integers
#[derive(
	Default,
	Clone,
	Encode,
	Decode,
	PartialEq,
	Debug,
	TypeInfo,
	Eq,
	MaxEncodedLen,
	Serialize,
	Deserialize,
)]
pub struct ParquetInteger {
	bit_width: u8,

	// Note: If we want the default to be signed/unsigned, we should add an
	// explicit default value here:
	// #[derivative(Default(value = "false"))]
	sign: bool,
}

/// Parquet Decimals
#[derive(
	Default,
	Clone,
	Encode,
	Decode,
	PartialEq,
	Debug,
	TypeInfo,
	Eq,
	MaxEncodedLen,
	Serialize,
	Deserialize,
)]
pub struct ParquetDecimal {
	scale: u8,

	// Note: in theory, precision is unbounded. But a u8 should be fine for practical cases
	precision: u8,
}
