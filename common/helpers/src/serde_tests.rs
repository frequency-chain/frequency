//use serde::serde_json_core;

const VALID_EXAMPLES: &[&str] = &[
	(r#"{
        "name": "John Doe",
        "age": 43,
        "address": {
            "street": "10 Downing Street",
            "city": "London"
        },
        "phones": [
            "+44 1234567",
            "+44 2345678"
        ]
    }"#),
];

#[test]
fn test_validate_json_schema() {
	for (raw_schema, expected) in VALID_EXAMPLES {
		let schema_result = serde_json::validate_json_schema(raw_schema);
		if *expected {
			assert!(
				schema_result.is_ok(),
				"schema {} was supposed to be valid; error: {:?}",
				raw_schema,
				schema_result.err()
			);
		} else {
			assert!(
				schema_result.is_err(),
				"schema {} was supposed to be invalid; error: {:?}",
				raw_schema,
				schema_result.err()
			);
		}
	}
}

