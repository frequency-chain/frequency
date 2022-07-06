use serde_json_core::{from_slice, de::{Error, IgnoredAny}};
use sp_std::vec::Vec;
use frame_support::{dispatch::{DispatchResult, DispatchError},assert_ok};


// #[derive(Debug)]
pub enum SerdeError {
	InvalidSchema(),
}


pub fn validate_json_model(json_schema: Vec<u8>) -> Result<(), Error> {
	// let result: (()), usize)  = from_slice(&json_schema).map_err(|e| SerdeError::InvalidSchema(e))?;
	let result = from_slice::<()>(&json_schema)?;

	Ok(())
	// println!("{}",_result.unwrap());
	// Ok(())
	// .map_err(|_| SerdeError::InvalidSchema()); // map error
	// match result {
	// 	Value::Null => Err(SerdeError::InvalidNullSchema()),
	// 	_ => Ok(()),
	// }
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
		assert_ok!(validate_json_model(create_schema_vec(test_str_raw)));
	}
}

#[test]
fn serde_helper_invalid_schema() {
	for test_str_raw in [
		r#"{"name","John Doe"}"#,
		r#"{"minimum": -90, 90}"#,
		r#"{"fruits":[ "apple",{"fruitName": "orange" "fruitLike": true }}"#,
	] {
		assert_ok!(validate_json_model(create_schema_vec(test_str_raw)));
	}
}

// #[test]
// fn serde_helper_null_schema() {
// 	let bad_schema = r#"{""}"#;
// 	let result = validate_json_model(create_schema_vec(bad_schema));
// 	assert!(result.is_err());
// }

#[test]
fn serde_helper_utf8_encoding_schema() {
	let bad_schema = r#"{"a":"Espíritu navideño"}"#;
	let result = validate_json_model(create_schema_vec(bad_schema));
	assert_ok!(result);
}
