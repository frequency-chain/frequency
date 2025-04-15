use serde::{Deserialize, Serialize};

/// Parquet numeric types: <https://github.com/apache/parquet-format/blob/master/LogicalTypes.md>
#[derive(Clone, PartialEq, Debug, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ParquetNumericType {
	/// Integers
	Integer(ParquetInteger),
	/// Decimals
	Decimal(ParquetDecimal),
}

/// Parquet Integers
#[derive(Clone, PartialEq, Debug, Eq, Serialize, Deserialize)]
pub struct ParquetInteger {
	/// The number of bits allocated to the parquet integer
	pub bit_width: u8,
	/// Whether the integer is signed
	pub sign: bool,
}

/// Parquet Decimals
#[derive(Clone, PartialEq, Debug, Eq, Serialize, Deserialize)]
pub struct ParquetDecimal {
	scale: u8,

	// Note: in theory, precision is unbounded. But a u8 should be fine for practical cases
	precision: u8,
}
