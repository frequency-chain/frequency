use serde::{Deserialize, Serialize};

/// Parquet String types: <https://github.com/apache/parquet-format/blob/master/LogicalTypes.md>
/// NOTE: We are not supporting ENUMs yet.

#[derive(Clone, PartialEq, Debug, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ParquetStringType {
	/// Parquet strings
	String,
	/// Parquet UUIDs
	UUID,
}
