#[cfg(test)]
use super::mock::*;

#[allow(unused_imports)]
use frame_support::assert_noop;
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

#[allow(dead_code)]
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
		"true",
		"567",
		r#"string"#,
		"",
		r#"["this","is","a","weird","array"],
		r#"{"name","John Doe"}"#,
		r#"{"minimum": -90, 90}"#,
		r#"{"fruits":[ "apple",{"fruitName": "orange" "fruitLike": true }}"#,
	] {
		assert!(validate_json_model(create_schema_vec(test_str_raw)).is_err());
	}
}

#[test]
fn serde_helper_deserialzer_error() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			validate_json_model(create_schema_vec(r#"{"name":"#)),
			SerdeError::DeserializationError
		);
	});
}

#[test]
fn serde_helper_null_schema() {
	new_test_ext().execute_with(|| {
		assert_noop!(validate_json_model(create_schema_vec("null")), SerdeError::InvalidNullSchema);
	});
}

#[test]
fn serde_helper_utf8_encoding_schema() {
	let utf8_schema = r#"{"a":"Espíritu navideño"}"#;
	let result = validate_json_model(create_schema_vec(utf8_schema));
	assert!(result.is_ok());
}
