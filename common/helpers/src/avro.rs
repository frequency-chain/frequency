use crate::types::*;
use apache_avro::{from_avro_datum, schema::Schema, to_avro_datum, types::Record, Codec, Writer};
use std::{collections::HashMap, io::Cursor, str};

/// Represents error types returned by the `avro` module.
#[derive(thiserror::Error, Debug)]
pub enum AvroError {
	#[error("I/O error")]
	Io(#[from] std::io::Error),
	#[error("Avro error")]
	Avro(#[from] apache_avro::Error),
	#[error("Invalid avro schema: {0}")]
	InvalidSchema(String),
	#[error("Invalid avro records")]
	InvalidRecords(),
}

/// Function to convert a raw schema into serialized Avro schema.
/// If schema is malformed or invalid, returns an error.
///
/// # Examples
/// ```
/// use common_helpers::avro;
/// use common_helpers::types::*;
/// let raw_schema = r#"{"type": "record", "name": "User", "fields": [{"name": "name", "type": "string"}, {"name": "favorite_number", "type": "int"}]}"#;
/// let schema_result = avro::fingerprint_raw_schema(raw_schema);
/// assert!(schema_result.is_ok());
/// let serialized_schema = schema_result.unwrap().1;
/// ```
pub fn fingerprint_raw_schema(raw_schema: &str) -> Result<(Schema, Vec<u8>), AvroError> {
	let schema_result = Schema::parse_str(raw_schema)?;
	let schema_canonical_form = schema_result.canonical_form();
	Ok((schema_result, schema_canonical_form.as_bytes().to_vec()))
}

///Function to convert a serialized Avro schema into Avro Schema type.
/// If schema is malformed or invalid, returns an error.
///
/// # Examples
/// ```
/// use common_helpers::avro;
/// use common_helpers::types::*;
/// let raw_schema = r#"{"type": "record", "name": "User", "fields": [{"name": "name", "type": "string"}, {"name": "favorite_number", "type": "int"}]}"#;
/// let serialized_schema = avro::fingerprint_raw_schema(raw_schema);
/// assert!(serialized_schema.is_ok());
/// let schema = serialized_schema.unwrap().1;
/// let translated_schema = avro::translate_schema(schema);
/// assert!(translated_schema.is_ok());
/// ```
pub fn translate_schema(serialized_schema: Vec<u8>) -> Result<Schema, AvroError> {
	match str::from_utf8(&serialized_schema) {
		Ok(schema_str) => {
			let schema = Schema::parse_str(schema_str)?;
			Ok(schema)
		},
		Err(error) => Err(AvroError::InvalidSchema(error.to_string())),
	}
}

/// Function to get the schema writer with default container as `Vec<u8>`
///
/// # Examples
/// ```
/// use common_helpers::avro;
/// use common_helpers::types::*;
/// let raw_schema = r#"{"type": "record", "name": "User", "fields": [{"name": "name", "type": "string"}, {"name": "favorite_number", "type": "int"}]}"#;
/// let schema_result = avro::fingerprint_raw_schema(raw_schema);
/// assert!(schema_result.is_ok());
/// let avro_schema = schema_result.unwrap().0;
/// let schema_writer = avro::get_schema_data_writer(&avro_schema);
/// ```
pub fn get_schema_data_writer(schema: &Schema) -> Writer<Vec<u8>> {
	Writer::with_codec(schema, Vec::new(), Codec::Snappy)
}

/// Function to populate a given schema with data and return serialized record.
///
/// # Remarks
/// * `record` is the record to be written as HashMap<String, SchemaValue> where SchemaValue is common types
///
/// # Examples
/// ```
/// use common_helpers::avro;
/// use common_helpers::types::*;
/// use std::{collections::HashMap};
/// let raw_schema = r#"
/// {
///     "type": "record",
///     "name": "test",
///     "fields": [
///         {"name": "a", "type": "long", "default": 42},
///         {"name": "b", "type": "string"}
///     ]
/// }
/// "#;
/// let schema_fingerprint = avro::fingerprint_raw_schema(raw_schema);
/// let mut hashmap_data = HashMap::new();
/// let mut name = "name".to_string();
/// let mut name_field = SchemaValue::String("John".to_string());
/// hashmap_data.insert("a".to_string(), SchemaValue::Long(27i64));
/// hashmap_data.insert("b".to_string(), SchemaValue::String("foo".to_string()));
/// assert!(schema_fingerprint.is_ok());
/// let data_schema = schema_fingerprint.unwrap().0;
/// let serialized_record = avro::populate_schema_and_serialize(&data_schema, &hashmap_data);
/// assert!(serialized_record.is_ok());
/// ```
pub fn populate_schema_and_serialize(
	schema: &Schema,
	records: &HashMap<String, SchemaValue>,
) -> Result<Vec<u8>, AvroError> {
	let writer = get_schema_data_writer(schema);
	match Record::new(writer.schema()) {
		None =>
			Err(AvroError::InvalidSchema("Could not create record from this schema".to_string())),
		Some(mut record_list) => {
			for (field_name, field_value) in records.iter() {
				record_list.put(field_name, field_value.clone());
			}
			let datum_res = to_avro_datum(schema, record_list)?;
			Ok(datum_res)
		},
	}
}

/// Function to get serialized datum data for a given schema into hashmap.
///
/// # Examples
/// ```
/// use common_helpers::avro;
/// use common_helpers::types::*;
/// use std::{collections::HashMap};
/// let raw_schema = r#"
/// {
///    "type": "record",
///   "name": "test",
///  "fields": [
///    {"name": "a", "type": "long", "default": 42},
///   {"name": "b", "type": "string"}
/// ]
/// }
/// "#;
/// let schema_fingerprint = avro::fingerprint_raw_schema(raw_schema);
/// let mut hashmap_data = HashMap::new();
/// let mut name = "name".to_string();
/// let mut name_field = SchemaValue::String("John".to_string());
/// hashmap_data.insert("a".to_string(), SchemaValue::Long(27i64));
/// hashmap_data.insert("b".to_string(), SchemaValue::String("foo".to_string()));
/// assert!(schema_fingerprint.is_ok());
/// let data_schema = schema_fingerprint.unwrap().0;
/// let serialized_record = avro::populate_schema_and_serialize(&data_schema, &hashmap_data);
/// assert!(serialized_record.is_ok());
/// let serialized_data = serialized_record.unwrap();
/// let deserialized_data = avro::get_schema_data_map(&serialized_data, &data_schema);
/// assert!(deserialized_data.is_ok());
/// ```
pub fn get_schema_data_map<'a>(
	serialized_data: &'a Vec<u8>,
	schema: &'a Schema,
) -> Result<HashMap<String, SchemaValue>, AvroError> {
	let from_data_datum = from_avro_datum(schema, &mut Cursor::new(serialized_data), None)?;
	let mut result_record = HashMap::<String, SchemaValue>::new();

	match from_data_datum {
		SchemaValue::Record(record) =>
			for (field_name, field_value) in record.iter() {
				result_record.insert(field_name.clone(), field_value.clone());
			},
		_ => return Err(AvroError::InvalidRecords()),
	}

	Ok(result_record)
}

/// Function to validate incoming json serialized schema against avro schema.
///
/// # Examples
/// ```
/// use common_helpers::avro;
/// use common_helpers::types::*;
/// let raw_schema = r#"
/// {
///    "type": "record",
///   "name": "test",
///  "fields": [
///    {"name": "a", "type": "long", "default": 42},
///   {"name": "b", "type": "string"}
/// ]
/// }
/// "#;
/// let schema_fingerprint = avro::fingerprint_raw_schema(raw_schema);
/// assert!(schema_fingerprint.is_ok());
/// ```
pub fn validate_raw_avro_schema(json_schema: &Vec<u8>) -> Result<(), AvroError> {
	match String::from_utf8(json_schema.clone()) {
		Err(_e) => Err(AvroError::InvalidSchema("Invalid schema".to_string())),
		Ok(avro_schema) => {
			let schema_fingerprint = fingerprint_raw_schema(&avro_schema);
			if schema_fingerprint.is_err() {
				return Err(AvroError::InvalidSchema("Invalid schema".to_string()))
			}
			Ok(())
		},
	}
}
