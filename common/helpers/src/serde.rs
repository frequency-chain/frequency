use serde_json;
// use serde_json_core;
// use serde::de::{Err, Error};

// Represents error types returned by the `serde` module.
#[derive(thiserror::Error, Debug)]
pub enum SerdeError {
	#[error("Invalid JSON schema: {0}")]
	InvalidSchema(String),
	#[error("Invalid JSON records")]
	InvalidRecords(),
}

pub fn validate_json_schema(json_schema: Vec<u8>) -> Result<(), SerdeError> {
    let schema_raw: Result<Vec<u8>, serde_json::Error> = serde_json::from_slice(&json_schema);
	if schema_raw.is_err() {
		return Err(SerdeError::InvalidSchema("Invalid schema".to_string()))
	}
	Ok(())
}