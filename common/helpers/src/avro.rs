use crate::types::*;
use apache_avro::{schema::Schema, types::Record, Codec, Reader, Writer};
use std::{collections::HashMap, panic, str};

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
/// # Arguments
/// * `raw_schema` - raw schema to be converted
/// # Returns
/// * `Result<(Schema, Vec<u8>), AvroError>` - structured and serialized Avro schema
/// # Examples
/// ```
/// use avro::*;
/// let raw_schema = r#"{"type": "record", "name": "User", "fields": [{"name": "name", "type": "string"}, {"name": "favorite_number", "type": "int"}]}"#;
/// let schema_result = fingerprint_raw_schema(raw_schema);
/// assert!(schema_result.is_ok());
/// let serialized_schema = schema_result.unwrap().1;
pub fn fingerprint_raw_schema(raw_schema: &str) -> Result<(Schema, Vec<u8>), AvroError> {
	// parse_str will fail if schema is not valid
	panic::catch_unwind(|| {
		let schema_result = Schema::parse_str(raw_schema);
		// match the result of the parse_str
		match schema_result {
			Ok(schema) => {
				// get the fingerprint of the schema
				let schema_canonical_form = schema.canonical_form();
				// return the schema and the fingerprint
				Ok((schema, schema_canonical_form.as_bytes().to_vec()))
			},
			Err(e) => Err(AvroError::Avro(e)),
		}
	})
	.unwrap_or_else(|_| {
		// if the unwind is caught, return an error
		Err(AvroError::InvalidSchema(raw_schema.to_string()))
	})
}

/// Function to convert a list of raw schema into serialized Avro schema.
/// If schema is malformed or invalid, returns an error.
/// # Arguments
/// * `raw_schema` - raw schema list to be converted
/// # Returns
/// * `Result<(Vec<Schema>, Vec<Vec<u8>>), AvroError>` - structured and serialized Avro schemas
/// # Examples
/// ```
/// use avro::*;
/// let raw_schema = r#"{"type": "record", "name": "User", "fields": [{"name": "name", "type": "string"}, {"name": "favorite_number", "type": "int"}]}"#;
/// let schema_result = fingerprint_raw_schema_list(vec![raw_schema.to_string()]);
/// assert!(schema_result.is_ok());
/// let serialized_schemas = schema_result.unwrap().1;
/// ```
pub fn fingerprint_raw_schema_list(
	raw_schema: &[&str],
) -> Result<(Vec<Schema>, Vec<Vec<u8>>), AvroError> {
	let mut schema_fingerprints = Vec::new();
	let mut schemas = Vec::new();
	for s in raw_schema {
		let (schema, fingerprint) = fingerprint_raw_schema(s)?;
		schema_fingerprints.push(fingerprint);
		schemas.push(schema);
	}
	Ok((schemas, schema_fingerprints))
}

///Function to convert a serialized Avro schema into Avro Schema type.
/// If schema is malformed or invalid, returns an error.
/// # Arguments
/// * `serialized_schema` - serialized Avro schema to be converted
/// # Returns
/// * `Result<Schema, AvroError>` - structured Avro schema
/// # Examples
/// ```
/// use avro::*;
/// let raw_schema = r#"{"type": "record", "name": "User", "fields": [{"name": "name", "type": "string"}, {"name": "favorite_number", "type": "int"}]}"#;
/// let serialized_schema = fingerprint_raw_schema(raw_schema);
/// assert!(serialized_schema.is_ok());
/// let schema = schema_result.unwrap().1;
/// let translated_schema = translate_schema(&schema);
/// assert!(translated_schema.is_ok());
/// ```
pub fn translate_schema(serialized_schema: Vec<u8>) -> Result<Schema, AvroError> {
	let schema_str = str::from_utf8(&serialized_schema);
	match schema_str {
		Ok(schema_str) => {
			let schema = Schema::parse_str(schema_str)?;
			Ok(schema)
		},
		Err(error) => Err(AvroError::InvalidSchema(error.to_string())),
	}
}

///Function to convert a list of serialized Avro schema into Avro Schema type.
/// If schema is malformed or invalid, returns an error.
/// # Arguments
/// * `serialized_schema` - list of serialized Avro schema to be converted
/// # Returns
/// * `Result<Vec<Schema>, AvroError>` - structured Avro schema
/// # Examples
/// ```
/// use avro::*;
/// let raw_schema = r#"{"type": "record", "name": "User", "fields": [{"name": "name", "type": "string"}, {"name": "favorite_number", "type": "int"}]}"#;
/// let serialized_schema = fingerprint_raw_schema(raw_schema);
/// assert!(serialized_schema.is_ok());
/// let schema = schema_result.unwrap().1;
/// let translated_schema = translate_schemas(&vec![schema]);
/// assert!(translated_schema.is_ok());
/// ```
pub fn translate_schemas(serialized_schema: Vec<Vec<u8>>) -> Result<Vec<Schema>, AvroError> {
	let mut schemas = Vec::new();
	for s in serialized_schema {
		schemas.push(translate_schema(s)?);
	}
	Ok(schemas)
}

/// Function to get the schema writer with default container as Vec<u8>
/// # Examples
/// ```
/// use avro::*;
/// let writer = get_schema_data_writer();
/// ```
pub fn get_schema_data_writer<'a>(schema: &'a Schema) -> Writer<'a, Vec<u8>> {
	Writer::with_codec(schema, Vec::new(), Codec::Snappy)
}

/// Function to populate a given schema writer with schema data.
/// # Arguments
/// * `writer` - io Writer ( can be obtained from get_schema_data_writer() )
/// * `records` - record to be written as HashMap<String, SchemaValue> where SchemaValue is common types
/// # Examples
/// ```
/// use avro::*;
/// let writer = get_schema_data_writer();
/// let record = vec![("name".to_string(), SchemaValue::String("John".to_string())), ("favorite_number".to_string(), SchemaValue::Int(42))];
/// populate_records(writer, record);
/// ```
pub fn populate_records(
	writer: &mut Writer<Vec<u8>>,
	records: &HashMap<String, SchemaValue>,
) -> Result<(), AvroError> {
	let mut record_list = Record::new(writer.schema()).unwrap();
	for (field_name, field_value) in records.iter() {
		record_list.put(field_name, field_value.clone());
	}
	writer.append(record_list)?;
	Ok(())
}

/// Function to populate a given schema with data and return serialized record.
/// # Arguments
/// * `schema` - Avro schema
/// * `record` - record to be written as HashMap<String, SchemaValue> where SchemaValue is common types
/// # Examples
/// ```
/// use avro::*;
/// let schema = r#"{"type": "record", "name": "User", "fields": [{"name": "name", "type": "string"}, {"name": "favorite_number", "type": "int"}]}"#;
/// let record = vec![("name".to_string(), SchemaValue::String("John".to_string())), ("favorite_number".to_string(), SchemaValue::Int(42))];
/// let schema_fingerprint = fingerprint_raw_schema(schema);
/// assert!(schema_fingerprint.is_ok());
/// let serialized_record = populate_schema_and_serialize(schema_fingerprint.unwrap().1, record);
pub fn populate_schema_and_serialize(
	schema: &Schema,
	record: &HashMap<String, SchemaValue>,
) -> Result<Vec<u8>, AvroError> {
	let mut writer = Writer::with_codec(schema, Vec::new(), Codec::Snappy);
	let mut record_list = Record::new(writer.schema()).unwrap();
	for (field_name, field_value) in record.iter() {
		record_list.put(field_name, field_value.clone());
	}
	writer.append(record_list)?;
	let serialized_bytes = writer.into_inner();
	match serialized_bytes {
		Ok(serialized_bytes) => Ok(serialized_bytes),
		Err(error) => Err(AvroError::Avro(error)),
	}
}

/// Function to populate a schema with serialized data and return an Avro reader.
/// # Arguments
/// * `schema` - Avro schema
/// * `serialized_data` - serialized data
/// # Examples
/// ```
/// use avro::*;
/// let schema = r#"{"type": "record", "name": "User", "fields": [{"name": "name", "type": "string"}, {"name": "favorite_number", "type": "int"}]}"#;
/// let record = vec![("name".to_string(), SchemaValue::String("John".to_string())), ("favorite_number".to_string(), SchemaValue::Int(42))];
/// let schema_fingerprint = fingerprint_raw_schema(schema);
/// assert!(schema_fingerprint.is_ok());
/// let serialized_record = populate_schema_and_serialize(schema_fingerprint.unwrap().1, record);
/// let reader = get_schema_data_reader(schema_fingerprint.unwrap().1, serialized_record.unwrap());
/// ```
pub fn get_schema_data_reader<'a>(
	serialized_data: &'a Vec<u8>,
	schema: &'a Schema,
) -> Result<Reader<'a, &'a [u8]>, AvroError> {
	Ok(Reader::with_schema(&schema, &serialized_data[..])?)
}

/// Function to read serialized data into a schema and return a hashmap of records(K,V).
/// # Arguments
/// * `schema` - Avro schema
/// * `serialized_data` - serialized data
/// # Returns
/// * `Result<HashMap<String, SchemaValue>, AvroError>` - HashMap of records
/// # Examples
/// ```
/// use avro::*;
/// let schema = r#"{"type": "record", "name": "User", "fields": [{"name": "name", "type": "string"}, {"name": "favorite_number", "type": "int"}]}"#;
/// let record = vec![("name".to_string(), SchemaValue::String("John".to_string())), ("favorite_number".to_string(), SchemaValue::Int(42))];
/// let schema_fingerprint = fingerprint_raw_schema(schema);
/// assert!(schema_fingerprint.is_ok());
/// let serialized_record = populate_schema_and_serialize(schema_fingerprint.unwrap().1, record);
/// let record_map = get_schema_data_map(schema_fingerprint.unwrap().1, serialized_record.unwrap());
pub fn get_schema_data_map<'a>(
    serialized_data: &'a Vec<u8>,
    schema: &'a Schema,
) -> Result<HashMap<String, SchemaValue>, AvroError> {
    let reader = get_schema_data_reader(serialized_data, schema)?;
    let mut result_record = HashMap::<String, SchemaValue>::new();
    for record in reader {
        let record_value = record.unwrap();
        match record_value {
            SchemaValue::Record(record) => {
                for (field_name, field_value) in record.iter() {
                    result_record.insert(field_name.clone(), field_value.clone());
                }
            }
            _ => {
                return Err(AvroError::InvalidRecords());
            }
        }
    }
    Ok(result_record)
}
