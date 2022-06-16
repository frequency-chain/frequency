// use serde_json_core;
use serde_json::{de, ser, Error, from_slice};

// Represents error types returned by the `serde` module.
 
#[derive(Error, Debug)]
pub enum SerdeError {
	#[error("Invalid JSON schema: {0}")]
	InvalidSchema(str),
	#[error("Invalid JSON records")]
	InvalidRecords(),
}

pub fn validate_json_schema(json_schema: Vec<u8>) -> Result<(), SerdeError> {
    let schema_raw: Result<Vec<u8>, serde_json::ser::Error> = serde_json::from_slice(&json_schema).unwrap();
	if schema_raw.is_err() {
		return Err(SerdeError::InvalidSchema("Invalid schema".to_string()))
	}
	Ok(())
}