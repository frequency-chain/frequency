// use serde_json_core;
use serde_json::{Result, Value, Error};

pub fn validate_json_schema(json_schema: Vec<u8>) -> Result<()> { // short curcuit
    let result: Value = serde_json::from_slice(&json_schema)?;
    match result {
        Value::Null => Error()
    }
    Ok(())
}

#[test]
fn validate_serde_helper() {
    let test_str_raw = r#"{"name":"John Doe"}"#;
    let result = validate_json_schema(Vec::from(test_str_raw.as_bytes()));
    println!("Here's the result {:?}", result);
    assert!(result.is_ok());
}
