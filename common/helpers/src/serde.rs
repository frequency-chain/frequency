use crate::types::*;
use serde::serde_json_core;

/// Represents error types returned by the `avro` module.
#[derive(thiserror::Error, Debug)]
pub enum SerdeError {
	#[error("Invalid JSON schema: {0}")]
	InvalidSchema(String),
	#[error("Invalid JSON records")]
	InvalidRecords(),
}

pub fn validate_JSON_schema(json_schema: &str) -> Result<(), SerdeError> {
    let schema_raw = serde_json::from_str(json_schema)?;
    if schema_raw.is_err() {
		return Err(SerdeError::InvalidSchema("Invalid schema".to_string()))
	}
	Ok(())
}