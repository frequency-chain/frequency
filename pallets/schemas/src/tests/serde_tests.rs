use crate::{serde::*, tests::mock::*};
use frame_support::assert_noop;

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
		r#"{ "links": {
			"self": "http://example.com/articles?page[number]=3&page[size]=1",
			"first": "http://example.com/articles?page[number]=1&page[size]=1"
			}}"#,
		r#"{ "alias": "0xd8f3" }"#,
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
			r#"{ "name", "John Doe" }"#,
		r#"{ "minimum": -90, 90 }"#,
		r#"{ "fruits": [ "apple", {"fruitName": "orange" "fruitLike": true }}"#,
	] {
		assert!(validate_json_model(create_schema_vec(test_str_raw)).is_err());
	}
}

#[test]
fn serde_helper_deserialzer_error() {
	new_test_ext().execute_with(|| {
		for test_str_raw in [
			r#"{ "name": "#,                          // ExpectedSomeValue
			r#"{ 56: "number" }"#,                    // KeyMustBeAString
			r#"{ "file address": "file path" \r\n}"#, // EofWhileParsingObject
			r#"{ "unicode code point": "\ud83f" }"#,  // InvalidUnicodeCodePoint
			                                          // r#"{ "v": 300e715100 }"#,              // NumberOutOfRange
		] {
			assert_noop!(
				validate_json_model(create_schema_vec(test_str_raw)),
				SerdeError::DeserializationError
			);
		}
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
