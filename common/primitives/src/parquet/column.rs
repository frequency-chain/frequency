/// The model for Parquet data
use scale_info::prelude::string::String;
use serde::{Deserialize, Serialize};

use crate::parquet::{column_compression_codec::ColumnCompressionCodec, types::ParquetType};

/// Encapsulation for a single Parquet column
#[derive(Clone, PartialEq, Debug, Eq, Serialize, Deserialize)]
pub struct ParquetColumn {
	/// The label for what this column represents
	name: String,
	/// Parquet type labels
	column_type: ParquetType,
	/// Compression for column
	compression: ColumnCompressionCodec,
	/// Whether or not to use a bloom filter
	bloom_filter: bool,
}

impl ParquetColumn {
	/// Creates instance of struct
	pub fn new(
		name: String,
		column_type: ParquetType,
		compression: ColumnCompressionCodec,
		bloom_filter: bool,
	) -> ParquetColumn {
		ParquetColumn { name, column_type, compression, bloom_filter }
	}
}
