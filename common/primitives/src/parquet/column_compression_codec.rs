use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

/// Compression codecs: https://github.com/apache/parquet-format/blob/master/LogicalTypes.md
#[derive(Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen, Serialize, Deserialize)]
pub enum ColumnCompressionCodec {
	/// The compression used in the Bloom filter
	/// NOTE: more research required here
  Uncompressed,
	Snappy,
	Gzip,
	Lzo,
	Brotli,
	ZSTD,
	LZ4_RAW
}

impl Default for ColumnCompressionCodec {
	fn default() -> Self {
		Self::Uncompressed
	}
}
