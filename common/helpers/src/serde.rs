// use serde_json_core;
use serde_json::{Result, Value, Error};

pub fn validate_json_schema(json_schema: Vec<u8>) -> Result<()> { // short curcuit
    let result: Value = serde_json::from_slice(&json_schema)?;
    Ok(())
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
	let bad_schema = r#"{"John Doe":}"#;
    let result = validate_json_schema(Vec::from(bad_schema.as_bytes()));
	assert!(result.is_err());
}
