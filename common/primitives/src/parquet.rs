use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

/// Importing all base types
pub mod base;
/// Importing all numeric types
pub mod numeric;
/// Importing all string types
pub mod string;
/// Importing all temporal types
pub mod temporal;
/// Importing all labels
pub mod types;
/// Importing all compression codec types
pub mod column_compression_codec;

use crate::parquet::types::ParquetType;
use crate::parquet::column_compression_codec::ColumnCompressionCodec;

/// The model for Parquet data
#[derive(Default, Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen, Serialize, Deserialize)]
pub struct ParquetModel {
  /// Parquet type labels
  _type: ParquetType,
  /// Compression for column
  compression: ColumnCompressionCodec,
  /// Whether or not to use a bloom filter
  bloom_filter: bool,
}

impl ParquetModel {
  /// Creates instance of struct
  pub fn new(_type: ParquetType, compression: ColumnCompressionCodec, bloom_filter: bool) -> ParquetModel {
    ParquetModel {
      _type,
      compression,
      bloom_filter
    }
  }
}
