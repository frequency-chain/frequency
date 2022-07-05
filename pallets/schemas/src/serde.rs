use serde_json::{from_slice, Value};

#[derive(Debug)]
pub enum SerdeError {
	InvalidNullSchema(String),
	InvalidSchema(String),
}

pub fn validate_json_schema(json_schema: Vec<u8>) -> Result<(), SerdeError> {
	let result: Value =
		from_slice(&json_schema).map_err(|e| SerdeError::InvalidSchema(e.to_string()))?; // map error
	match result {
		Value::Null => Err(SerdeError::InvalidNullSchema("Provided schema is null".to_string())),
		_ => Ok(()),
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
