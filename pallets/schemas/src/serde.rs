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

fn create_schema_vec(from_string: &str) -> Vec<u8> {
	Vec::from(from_string.as_bytes())
}

#[test]
fn validate_serde_helper() {
	for test_str_raw in [
		r#"{"name":"John Doe"}"#,
		r#"{"minimum": -90,"maximum": 90}"#,
		r#"{"a":0}"#
	] {
		assert!(validate_json_schema(create_schema_vec(test_str_raw)).is_ok());
	}
}

#[test]
fn serde_helper_invalid_schema() {
	let bad_schema = r#"{"John Doe","nothing"}"#;
	let result = validate_json_schema(create_schema_vec(bad_schema));
	assert!(result.is_err());
}

#[test]
fn serde_helper_null_schema() {
	let bad_schema = r#"{""}"#;
	let result = validate_json_schema(create_schema_vec(bad_schema));
	assert!(result.is_err());
}

#[test]
fn serde_helper_utf8_encoding() {
	let bad_schema = r#"{"a":"Espíritu navideño"}"#;
	let result = validate_json_schema(create_schema_vec(bad_schema));
	assert!(result.is_ok());
}
