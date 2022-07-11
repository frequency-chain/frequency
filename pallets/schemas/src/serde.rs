use serde_json_core::from_slice;
use sp_std::vec::Vec;

pub fn is_valid_schema(json_schema: Vec<u8>) -> bool {

	// The given json is wrapped with "{ }"
	match json_schema.get(0) {
		Some(123) => true, // Curly left "{"
		_ => {return false},
	};

	match json_schema.get(json_schema.len() - 1) {
		Some(125) =>true, // Curly right "}"
		_ => {return false},
	};

	match from_slice::<()>(&json_schema)

	// .map_or_else(|_| false, |_| true)
	// map(|_| true).or_else(false).unwrap()
	{
		Ok(_) => true,
		value => { println!( "the value that fails: {:?}", value); return false},
	}

}

#[allow(dead_code)]
fn create_schema_vec(from_string: &str) -> Vec<u8> {
	Vec::from(from_string.as_bytes())
}

#[test]
fn serde_helper_valid_schema() {
	for test_str_raw in [
		r#"{"type": "string", "name": "John Doe"}"#,
		r#"{"minimum": -90,"maximum": 90}"#,
		r#"{"a":0}"#,
		r#"{"fruits":[ "apple",{"fruitName": "orange","fruitLike": true }]}"#,
	] {
		assert!(is_valid_schema(create_schema_vec(test_str_raw)), " Failed test string {}", test_str_raw);
	}
}

#[test]
fn serde_helper_invalid_schema() {
	for test_str_raw in [
		r#"{"name","John Doe"}"#,
		r#"{"minimum": -90, 90}"#,
		r#"{"fruits":[ "apple",{"fruitName": "orange" "fruitLike": true }}"#,
		"true",
		"",
		r#"["apple",{"fruitName": "orange","fruitLike": true }]"#,
		"5",
		r#"{"hello""#,
		r#"56}"#,
		r#" { "type": "bool" } "#
	] {
		assert_eq!(is_valid_schema(create_schema_vec(test_str_raw)), false);
	}
}

#[test]
fn serde_helper_null_schema() {
	let bad_schema = r#"{""}"#;
	let result = is_valid_schema(create_schema_vec(bad_schema));
	assert_eq!(result, false);
}

#[test]
fn serde_helper_utf8_encoding_schema() {
	let utf_schema = r#"{"a":"Espíritu navideño"}"#;
	let result = is_valid_schema(create_schema_vec(utf_schema));
	assert_eq!(result, true);
}
