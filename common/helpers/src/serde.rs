// use serde_json_core;
use serde_json::{Result};

pub fn validate_json_schema(json_schema: Vec<u8>) -> Result<()> {
    let slice_json_schema = json_schema.as_slice();
    println!("Sliced json schema in Serde: {:?}", slice_json_schema);
    serde_json::from_slice(json_schema.as_slice())
}

#[test]
fn validate_serde_helper() {
    let test_str_raw = r#"{"name":"John Doe"}"#;
    let result = validate_json_schema(Vec::from(test_str_raw.as_bytes()));
    println!("Here's the result {:?}", result);
    assert_eq!(result.unwrap(),  ());
}
