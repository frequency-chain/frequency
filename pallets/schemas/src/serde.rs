use serde_json::{from_slice, Value};
use sp_std::vec::Vec;

#[derive(Debug)]
pub enum SerdeError {
	InvalidNullSchema(),
	InvalidSchema(),
}

pub fn validate_json_model(json_schema: Vec<u8>) -> Result<(), SerdeError> {
	let result: Value =
		from_slice(&json_schema).map_err(|_| SerdeError::InvalidSchema())?; // map error
	match result {
		Value::Null => Err(SerdeError::InvalidNullSchema()),
		_ => Ok(()),
	}
}

fn create_schema_vec(from_string: &str) -> Vec<u8> {
	Vec::from(from_string.as_bytes())
}

#[test]
fn serde_helper_valid_schema() {
	for test_str_raw in [
		r#"{"name":"John Doe"}"#,
		r#"{"minimum": -90,"maximum": 90}"#,
		r#"{"a":0}"#,
		r#"{"fruits":[ "apple",{"fruitName": "orange","fruitLike": true }]}"#,
	] {
		assert!(validate_json_model(create_schema_vec(test_str_raw)).is_ok());
	}
}

#[test]
fn serde_helper_invalid_schema() {
	for test_str_raw in [
		r#"{"name","John Doe"}"#,
		r#"{"minimum": -90, 90}"#,
		r#"{"fruits":[ "apple",{"fruitName": "orange" "fruitLike": true }}"#,
	] {
		assert!(validate_json_model(create_schema_vec(test_str_raw)).is_err());
	}
}

#[test]
fn serde_helper_null_schema() {
	let bad_schema = r#"{""}"#;
	let result = validate_json_model(create_schema_vec(bad_schema));
	assert!(result.is_err());
}

#[test]
fn serde_helper_utf8_encoding_schema() {
	let bad_schema = r#"{"a":"Espíritu navideño"}"#;
	let result = validate_json_model(create_schema_vec(bad_schema));
	assert!(result.is_ok());
}
