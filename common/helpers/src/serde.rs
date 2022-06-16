// use serde_json_core;
use serde_json::{Result};

pub fn validate_json_schema(json_schema: Vec<u8>) -> Result<()> {
    serde_json::from_slice(&json_schema)
}
