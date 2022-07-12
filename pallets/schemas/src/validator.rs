use sp_std::vec::Vec;

pub fn is_valid_json(json_schema: Vec<u8>) -> bool {
	// The given json is wrapped with "{ }"
	let t = (json_schema.first(), json_schema.last());
	match t {
		(Some(123), Some(125)) => true,
		_ => false
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
		assert!(is_valid_json(create_schema_vec(test_str_raw)), " Failed test string {}", test_str_raw);
	}
}

#[test]
fn serde_helper_invalid_schema() {
	for test_str_raw in [
		"true",
		"",
		r#"["apple",{"fruitName": "orange","fruitLike": true }]"#,
		"5",
		r#"{"hello""#,
		r#"56}"#,
		r#" { "type": "bool" } "#
	] {
		assert_eq!(is_valid_json(create_schema_vec(test_str_raw)), false);
	}
}

#[test]
fn serde_helper_null_schema() {
	let null_schema = r#"{""}"#;
	let result = is_valid_json(create_schema_vec(null_schema));
	assert_eq!(result, true);
}

#[test]
fn serde_helper_utf8_encoding_schema() {
	let utf_schema = r#"{"a":"Espíritu navideño"}"#;
	let result = is_valid_json(create_schema_vec(utf_schema));
	assert_eq!(result, true);
}
