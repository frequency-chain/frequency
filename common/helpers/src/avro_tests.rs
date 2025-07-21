use std::collections::HashMap;

use crate::avro;
use apache_avro::types::Record;

pub type SchemaValue = apache_avro::types::Value;

const VALID_SCHEMAS: [&str; 19] = [
	r#""null""#,
	r#"{"type": "null"}"#,
	r#""boolean""#,
	r#"{"type": "boolean"}"#,
	r#""string""#,
	r#"{"type": "string"}"#,
	r#""bytes""#,
	r#"{"type": "bytes"}"#,
	r#""int""#,
	r#"{"type": "int"}"#,
	r#""long""#,
	r#"{"type": "long"}"#,
	r#""float""#,
	r#"{"type": "float"}"#,
	r#""double""#,
	r#"{"type": "double"}"#,
	r#"
		{
			"type": "record",
			"name": "test",
			"fields": [
				{"name": "a", "type": "long", "default": 42},
				{"name": "b", "type": "string"}
			]
		}
		"#,
	r#"{"type": "fixed", "name": "Test", "size": 1}"#,
	r#"{
                "type": "fixed",
                "name": "MyFixed",
                "namespace": "org.apache.hadoop.avro",
                "size": 1
            }"#,
];

const INVALID_SCHEMAS: [&str; 8] = [
	r#"{"type": "fixed", "name": "MissingSize"}"#,
	r#""{}"#,
	r#"true"#,
	r#""true""#,
	r#"{"type": "panther"}"#,
	r#"{"no_type": "test"}"#,
	r#"{"type": "fixed", "size": 314}"#,
	r#"
		{
			"type": "record",
			"name": "no_size_field",
			"fields": [
				{"name": "a"},
				{"name": "b"}
			]
		}
		"#,
];

#[test]
fn test_fingerprint_valid() {
	for valid_schema in VALID_SCHEMAS {
		let schema_result = avro::fingerprint_raw_schema(valid_schema);
		assert!(
			schema_result.is_ok(),
			"schema {} was supposed to be valid; error: {:?}",
			valid_schema,
			schema_result.err()
		);
	}
}

#[test]
fn test_fingerprint_invalid() {
	for invalid_schema in INVALID_SCHEMAS {
		let schema_result = avro::fingerprint_raw_schema(invalid_schema);
		assert!(
			schema_result.is_err(),
			"schema {} was supposed to be invalid; error: {:?}",
			invalid_schema,
			schema_result.err()
		);
	}
}

#[test]
/// Test schema round trip: raw Avro schema -> serialized Avro -> raw Avro Schema.
fn test_fingerprint_raw_schema_has_valid_cast_to_string_after_parse() {
	for valid_schema in VALID_SCHEMAS {
		let schema_result = avro::fingerprint_raw_schema(valid_schema);
		assert!(
			schema_result.is_ok(),
			"schema {} was supposed to be valid; error: {:?}",
			valid_schema,
			schema_result.err()
		);
		let (schema, schema_vec) = schema_result.unwrap();

		let translate_schema = avro::translate_schema(schema_vec);
		assert!(
			translate_schema.is_ok(),
			"schema {} was supposed to be valid; error: {:?}",
			valid_schema,
			translate_schema.err()
		);
		let translated_schema = translate_schema.unwrap();
		assert_eq!(translated_schema, schema);
	}
}

#[test]
fn test_fingerprint_raw_schema_errors_on_invalid_schema() {
	for invalid_schema in INVALID_SCHEMAS {
		let schema_result = avro::fingerprint_raw_schema(invalid_schema);
		assert!(
			schema_result.is_err(),
			"schema {} was supposed to be invalid; error: {:?}",
			invalid_schema,
			schema_result.err()
		);
	}
}

#[test]
fn test_get_writer_with_schema() {
	let schema_result = avro::fingerprint_raw_schema(r#"{"type": "int"}"#);
	assert!(schema_result.is_ok());
	let schema_res = schema_result.unwrap();
	let translate_schema = avro::translate_schema(schema_res.1);
	assert!(translate_schema.is_ok());
	let translated_schema = translate_schema.unwrap();
	let writer = avro::get_schema_data_writer(&translated_schema);
	assert_eq!(writer.schema(), &translated_schema);
}

#[test]
fn test_get_writer_with_data() {
	let raw_schema = r#"
    {
        "type": "record",
        "name": "test",
        "fields": [
            {"name": "a", "type": "long", "default": 42},
            {"name": "b", "type": "string"}
        ]
    }
    "#;
	let raw_schema_vec = raw_schema.as_bytes().to_vec();
	let schema_string = String::from_utf8(raw_schema_vec).unwrap();
	let schema_result = avro::fingerprint_raw_schema(&schema_string);
	assert!(schema_result.is_ok());
	let schema_res = schema_result.unwrap();
	let translate_schema = avro::translate_schema(schema_res.1);
	assert!(translate_schema.is_ok());
	let translated_schema = translate_schema.unwrap();
	let mut writer = avro::get_schema_data_writer(&translated_schema);
	assert_eq!(writer.schema(), &translated_schema);
	// the Record type models our Record schema
	let mut record = Record::new(writer.schema()).unwrap();
	record.put("a", 27i64);
	record.put("b", "foo");
	let result_write = writer.append(record);
	assert!(result_write.is_ok());
}

#[test]
fn test_set_writer_with_data() {
	let raw_schema = r#"
    {
        "type": "record",
        "name": "test",
        "fields": [
            {"name": "a", "type": "long", "default": 42},
            {"name": "b", "type": "string"}
        ]
    }
    "#;
	let schema_result = avro::fingerprint_raw_schema(raw_schema);
	assert!(schema_result.is_ok());
	let schema_res = schema_result.unwrap();
	let translate_schema = avro::translate_schema(schema_res.1);
	assert!(translate_schema.is_ok());
	let translated_schema = translate_schema.unwrap();
	let mut writer = avro::get_schema_data_writer(&translated_schema);
	assert_eq!(writer.schema(), &translated_schema);
	// the Record type models our Record schema
	let mut record = Record::new(writer.schema()).unwrap();
	record.put("a", 27i64);
	record.put("b", "foo");
	let result_write = writer.append(record);
	assert!(result_write.is_ok());
}

#[test]
fn test_populate_data_records() {
	let raw_schema = r#"
    {
        "type": "record",
        "name": "test",
        "fields": [
            {"name": "a", "type": "long", "default": 42},
            {"name": "b", "type": "string"}
        ]
    }
    "#;
	let schema_result = avro::fingerprint_raw_schema(raw_schema);
	assert!(schema_result.is_ok());
	let schema_res = schema_result.unwrap();
	let translate_schema = avro::translate_schema(schema_res.1);
	assert!(translate_schema.is_ok());
	let translated_schema = translate_schema.unwrap();
	let writer = avro::get_schema_data_writer(&translated_schema);
	assert_eq!(writer.schema(), &translated_schema);
	// hashmap to store the data
	let mut data_map = HashMap::new();
	// the Record type models our Record schema
	data_map.insert("a".to_string(), SchemaValue::Long(27i64));
	data_map.insert("b".to_string(), SchemaValue::String("foo".to_string()));

	let result_write = avro::populate_schema_and_serialize(&translated_schema, &data_map);
	assert!(result_write.is_ok());
}

#[test]
fn test_invalid_cast_to_string_after_parse() {
	for invalid_schema in INVALID_SCHEMAS {
		let actual = avro::fingerprint_raw_schema(invalid_schema);
		assert!(
			actual.is_err(),
			"schema {} was supposed to be invalid; error: {:?}",
			invalid_schema,
			actual.err()
		);
	}
}

#[test]
fn test_invalid_translation() {
	let bad_schema = "{\"something\": \"nothing\"}";
	let bad_bytes = bad_schema.as_bytes().to_vec();
	let schema_result = avro::translate_schema(bad_bytes);
	assert!(
		schema_result.is_err(),
		"schema {} was supposed to be invalid; error: {:?}",
		bad_schema,
		schema_result.err()
	);
}

#[test]
fn test_populate_data_serialized() {
	let raw_schema = r#"
    {
        "type": "record",
        "name": "test",
        "fields": [
            {"name": "a", "type": "long", "default": 42},
            {"name": "b", "type": "string"}
        ]
    }
    "#;
	let schema_result = avro::fingerprint_raw_schema(raw_schema);
	assert!(schema_result.is_ok());
	let schema_res = schema_result.unwrap();
	let translate_schema = avro::translate_schema(schema_res.1);
	assert!(translate_schema.is_ok());
	let translated_schema = translate_schema.unwrap();
	let writer = avro::get_schema_data_writer(&translated_schema);
	assert_eq!(writer.schema(), &translated_schema);
	// hashmap to store the data
	let mut data_map = HashMap::new();
	// the Record type models our Record schema
	data_map.insert("a".to_string(), SchemaValue::Long(27i64));
	data_map.insert("b".to_string(), SchemaValue::String("foo".to_string()));

	let result_write = avro::populate_schema_and_serialize(&translated_schema, &data_map);
	assert!(result_write.is_ok());
}

#[test]
fn test_reader_schema_with_data() {
	let raw_schema = r#"
    {
        "type": "record",
        "name": "test",
        "fields": [
            {"name": "a", "type": "long", "default": 42},
            {"name": "b", "type": "string"}
        ]
    }
    "#;
	let schema_result = avro::fingerprint_raw_schema(raw_schema);
	assert!(schema_result.is_ok());
	let schema_res = schema_result.unwrap();
	let translate_schema = avro::translate_schema(schema_res.1);
	assert!(translate_schema.is_ok());
	let translated_schema = translate_schema.unwrap();
	let writer = avro::get_schema_data_writer(&translated_schema);
	assert_eq!(writer.schema(), &translated_schema);
	// hashmap to store the data
	let mut data_map = HashMap::new();
	// the Record type models our Record schema
	data_map.insert("a".to_string(), SchemaValue::Long(27i64));
	data_map.insert("b".to_string(), SchemaValue::String("foo".to_string()));

	let result_write = avro::populate_schema_and_serialize(&translated_schema, &data_map);
	assert!(result_write.is_ok());
	let serialized_result = result_write.unwrap();
	let reader_res = avro::get_schema_data_map(&serialized_result, &translated_schema);
	assert!(reader_res.is_ok());
}

#[test]
fn test_end_to_end_flow() {
	// create a schema
	let raw_schema = r#"
    {
        "type": "record",
        "name": "test",
        "fields": [
            {"name": "a", "type": "long", "default": 42},
            {"name": "b", "type": "string"}
        ]
    }
    "#;
	let schema_result = avro::fingerprint_raw_schema(raw_schema);
	assert!(schema_result.is_ok());
	let schema_res = schema_result.unwrap();
	let translate_schema = avro::translate_schema(schema_res.1);
	assert!(translate_schema.is_ok());
	let translated_schema = translate_schema.unwrap();
	let writer = avro::get_schema_data_writer(&translated_schema);
	assert_eq!(writer.schema(), &translated_schema);
	// hashmap to store the data
	let mut data_map = HashMap::new();
	// the Record type models our Record schema
	data_map.insert("a".to_string(), SchemaValue::Long(27i64));
	data_map.insert("b".to_string(), SchemaValue::String("foo".to_string()));
	// write the data
	let result_write = avro::populate_schema_and_serialize(&translated_schema, &data_map);
	assert!(result_write.is_ok());
	let serialized_result = result_write.unwrap();
	// read the data
	let reader_res = avro::get_schema_data_map(&serialized_result, &translated_schema);
	assert!(reader_res.is_ok());
}

#[test]
fn test_end_to_end_flow_map() {
	// create a schema
	let raw_schema = r#"
    {
        "type": "record",
        "name": "test",
        "fields": [
            {"name": "a", "type": "long", "default": 42},
            {"name": "b", "type": "string"}
        ]
    }
    "#;
	let schema_result = avro::fingerprint_raw_schema(raw_schema);
	assert!(schema_result.is_ok());
	let schema_res = schema_result.unwrap();
	let translate_schema = avro::translate_schema(schema_res.1);
	assert!(translate_schema.is_ok());
	let translated_schema = translate_schema.unwrap();
	let writer = avro::get_schema_data_writer(&translated_schema);
	assert_eq!(writer.schema(), &translated_schema);
	// hashmap to store the data
	let mut data_map = HashMap::new();
	// the Record type models our Record schema
	data_map.insert("a".to_string(), SchemaValue::Long(27i64));
	data_map.insert("b".to_string(), SchemaValue::String("foo".to_string()));
	// write the data
	let result_write = avro::populate_schema_and_serialize(&translated_schema, &data_map);
	assert!(result_write.is_ok());
	let serialized_result = result_write.unwrap();
	let reader_res = avro::get_schema_data_map(&serialized_result, &translated_schema);
	assert!(reader_res.is_ok());
	let reader = reader_res.unwrap();
	assert_eq!(reader["a"], SchemaValue::Long(27i64));
	assert_eq!(reader["b"], SchemaValue::String("foo".to_string()));
}

#[test]
fn test_bad_records() {
	// create a schema
	let schema_result = avro::fingerprint_raw_schema(VALID_SCHEMAS[16]);
	assert!(schema_result.is_ok());
	let schema_res = schema_result.unwrap();
	let translate_schema = avro::translate_schema(schema_res.1);
	assert!(translate_schema.is_ok());
	let translated_schema = translate_schema.unwrap();
	let writer = avro::get_schema_data_writer(&translated_schema);
	assert_eq!(writer.schema(), &translated_schema);
	let serialized_result = vec![0u8];
	let reader_res = avro::get_schema_data_map(&serialized_result, &translated_schema);
	assert!(reader_res.is_err());
}

#[test]
fn test_json_serialized_avro_schema() {
	// create a schema
	let serialized_bytes = VALID_SCHEMAS[16].as_bytes().to_vec();
	let validation_res = avro::validate_raw_avro_schema(&serialized_bytes);
	assert!(validation_res.is_ok());
}

#[test]
fn test_json_serialized_bad_avro_schema() {
	let serialized_bytes = INVALID_SCHEMAS[6].as_bytes().to_vec();
	let validation_res = avro::validate_raw_avro_schema(&serialized_bytes);
	assert!(validation_res.is_err());
}
