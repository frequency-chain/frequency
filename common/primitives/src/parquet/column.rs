/// The model for Parquet data
use scale_info::TypeInfo;
use scale_info::prelude::string::String;
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

use crate::parquet::types::ParquetType;
use crate::parquet::column_compression_codec::ColumnCompressionCodec;

/// Encapsulation for a single Parquet column
#[derive(Default, Clone, PartialEq, Debug, Eq, Serialize, Deserialize)]
pub struct ParquetColumn {
  /// The label for what this column represents
  name: String,
  /// Parquet type labels
  _type: ParquetType,
  /// Compression for column
  compression: ColumnCompressionCodec,
  /// Whether or not to use a bloom filter
  bloom_filter: bool,
}

impl ParquetColumn {
  /// Creates instance of struct
  pub fn new(name: String, _type: ParquetType, compression: ColumnCompressionCodec, bloom_filter: bool) -> ParquetColumn {
    ParquetColumn {
      name,
      _type,
      compression,
      bloom_filter
    }
  }
}
