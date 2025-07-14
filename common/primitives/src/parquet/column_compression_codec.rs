use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};

/// Compression codecs: <https://github.com/apache/parquet-format/blob/master/LogicalTypes.md>
#[derive(
	Clone, Encode, Decode, PartialEq, Debug, TypeInfo, Eq, MaxEncodedLen, Serialize, Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ColumnCompressionCodec {
	/// Uncompressed
	Uncompressed,
	/// Snappy, aka Zippy, compression
	Snappy,
	/// Gzip compression
	Gzip,
	/// Lempel-Zip-Obenhumer compression: <https://en.wikipedia.org/wiki/Lempel%E2%80%93Ziv%E2%80%93Oberhumer>
	Lzo,
	/// Brotli compression
	Brotli,
	/// Zstandard compression: <https://en.wikipedia.org/wiki/Zstd>
	ZSTD,
	/// Lz4 compression without block headers
	Lz4Raw,
}

impl Default for ColumnCompressionCodec {
	fn default() -> Self {
		Self::Uncompressed
	}
}
