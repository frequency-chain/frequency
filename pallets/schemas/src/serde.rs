use serde_json::{from_slice, Value};
use sp_std::vec::Vec;

#[derive(Debug, PartialEq)]
pub enum SerdeError {
	InvalidNullSchema,
	InvalidSchema,
	DeserializationError,
}

pub fn validate_json_model(json_schema: Vec<u8>) -> Result<(), SerdeError> {
	let result: Value = from_slice(&json_schema).map_err(|_| SerdeError::DeserializationError)?;

	match result {
		Value::Null => Err(SerdeError::InvalidNullSchema),
		Value::Object(_) => Ok(()),
		_ => Err(SerdeError::InvalidSchema),
	}
}
