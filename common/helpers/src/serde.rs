// use serde_json_core;
use serde_json::{Error, de, ser};

// Represents error types returned by the `serde` module.
 
#[derive(Debug)]
pub enum SerdeError {
	InvalidSchema(String),
	InvalidRecords(String)
}

pub fn validate_json_schema(json_schema: Vec<u8>) -> Result<(), SerdeError> {
    let schema_raw: Result<Vec<u8>, Error> = serde_json::from_slice(&json_schema).unwrap();
	if schema_raw.is_err() {
		return Err(SerdeError::InvalidSchema("Invalid schema".to_string()))
	}
	Ok(())
}

impl From<serde_json::Error> for SerdeError {
    fn from(err: serde_json::Error) -> SerdeError {
        use serde_json::error::Category;
        match err.classify() {
            Category::Syntax => {
                SerdeError::InvalidSchema(err.to_string())
            }
            Category::Data | Category::Eof => {
                SerdeError::InvalidRecords(err.to_string())
            }
        }
    }
}