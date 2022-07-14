/// The model for Parquet data
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

use crate::parquet::types::ParquetType;
use crate::parquet::column_compression_codec::ColumnCompressionCodec;

/// Encapsulation for a single Parquet column
#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen, Serialize, Deserialize)]
pub struct ParquetColumn {
  /// Parquet type labels
  _type: ParquetType,
  /// Compression for column
  compression: ColumnCompressionCodec,
  /// Whether or not to use a bloom filter
  bloom_filter: bool,
}

impl ParquetColumn {
  /// Creates instance of struct
  pub fn new(_type: ParquetType, compression: ColumnCompressionCodec, bloom_filter: bool) -> ParquetColumn {
    ParquetColumn {
      _type,
      compression,
      bloom_filter
    }
  }
}
