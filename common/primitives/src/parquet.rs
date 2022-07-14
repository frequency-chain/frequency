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
/// Importing column type
pub mod column;

use crate::parquet::column::ParquetColumn;

/// Type for Parquet files. Files are just lists of columns.
pub type ParquetModel = Vec<ParquetColumn>;
