// use serde_json_core::{Result, Value, from_slice, Error};
use serde_json::{Value, from_slice, Error};

/// Represents error types returned by the `Serde` module.
#[derive(thiserror::Error, Debug)]
pub enum SerdeError {
    #[error("Serde error")]
	Serde(#[from] serde_json::Error),
    #[error("Invalid Null schema: {0}")]
	InvalidNullSchema(String),
	#[error("Invalid Json schema: {0}")]
	InvalidSchema(String),
	#[error("Invalid Json records")]
	InvalidRecords(),
}

pub fn validate_json_schema(json_schema: Vec<u8>) -> Result<()> {
    let result: Value = from_slice(&json_schema).map_err(|_| SerdeError::InvalidNullSchema("The provided Json schema is null"))?; // map error
    match result {
        Value::Null => Error{},
        _ => Ok(())
        // Ok() => Ok(()),
        // Err(error) => Err(SerdeError::InvalidSchema(error.to_string()))
    }
}

#[test]
fn validate_serde_helper() {
    let test_str_raw = r#"{"name":"John Doe"}"#;
    let result = validate_json_schema(Vec::from(test_str_raw.as_bytes()));
    assert!(result.is_ok());
}

#[test]
fn serde_helper_invalid_schema() {
	let bad_schema = r#"{"John Doe","nothing"}"#;
    let result = validate_json_schema(Vec::from(bad_schema.as_bytes()));
	assert!(result.is_err());
}
#[test]
fn serde_helper_null_schema() {
	let bad_schema = r#"{}"#;
    let result = validate_json_schema(Vec::from(bad_schema.as_bytes()));
	assert!(result.is_err());
}
