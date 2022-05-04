use crate::types::*;
use apache_avro::{from_avro_datum, schema::Schema, to_avro_datum, types::Record, Codec, Writer};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
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
/// # Arguments
/// * `raw_schema` - raw schema to be converted
/// # Returns
/// * `Result<(Schema, Vec<u8>), AvroError>` - structured and serialized Avro schema
/// # Examples
/// ```
/// use common_helpers::avro;
/// use common_helpers::types::*;
/// let raw_schema = r#"{"type": "record", "name": "User", "fields": [{"name": "name", "type": "string"}, {"name": "favorite_number", "type": "int"}]}"#;
/// let schema_result = avro::fingerprint_raw_schema(raw_schema);
/// assert!(schema_result.is_ok());
/// let serialized_schema = schema_result.unwrap().1;
pub fn fingerprint_raw_schema(raw_schema: &str) -> Result<(Schema, Vec<u8>), AvroError> {
	let schema_result = Schema::parse_str(raw_schema)?;
	let schema_canonical_form = schema_result.canonical_form();
	Ok((schema_result, schema_canonical_form.as_bytes().to_vec()))
}

/// Function to convert a list of raw schema into serialized Avro schema.
/// If schema is malformed or invalid, it is set to Null.
/// # Arguments
/// * `raw_schema` - raw schema list to be converted
/// # Returns
/// * `Result<(Vec<Schema>, Vec<Vec<u8>>), AvroError>` - structured and serialized Avro schemas
/// # Examples
/// ```
/// use common_helpers::avro;
/// use common_helpers::types::*;
/// let raw_schema = r#"{"type": "record", "name": "User", "fields": [{"name": "name", "type": "string"}, {"name": "favorite_number", "type": "int"}]}"#;
/// let vec_raw_schema: [&str; 1] = [raw_schema];
/// let schema_result = avro::fingerprint_raw_schema_list(&vec_raw_schema);
/// assert!(schema_result.is_ok());
/// let serialized_schemas = schema_result.unwrap().1;
/// ```
pub fn fingerprint_raw_schema_list(
	raw_schema: &[&str],
) -> Result<(Vec<Schema>, Vec<Vec<u8>>), AvroError> {
	let schemas: (Vec<Schema>, Vec<Vec<u8>>) = raw_schema
		.par_iter()
		.map(|r| -> (Schema, Vec<u8>) {
			let schema = fingerprint_raw_schema(r);
			match schema {
				Ok(schema) => schema,
				Err(_error) => (Schema::Null, r.to_string().as_bytes().to_vec()),
			}
		})
		.collect();

	Ok(schemas)
}

///Function to convert a serialized Avro schema into Avro Schema type.
/// If schema is malformed or invalid, returns an error.
/// # Arguments
/// * `serialized_schema` - serialized Avro schema to be converted
/// # Returns
/// * `Result<Schema, AvroError>` - structured Avro schema
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
/// If schema is malformed or invalid, it is set to Null.
/// # Arguments
/// * `serialized_schema` - list of serialized Avro schema to be converted
/// # Returns
/// * `Result<Vec<Schema>, AvroError>` - structured Avro schema
/// # Examples
/// ```
/// use common_helpers::avro;
/// use common_helpers::types::*;
/// let raw_schema = r#"{"type": "record", "name": "User", "fields": [{"name": "name", "type": "string"}, {"name": "favorite_number", "type": "int"}]}"#;
/// let serialized_schema = avro::fingerprint_raw_schema(raw_schema);
/// assert!(serialized_schema.is_ok());
/// let schema = serialized_schema.unwrap().1;
/// let vec_schema = vec![schema];
/// let translated_schema = avro::translate_schemas(vec_schema);
/// assert!(translated_schema.is_ok());
/// ```
pub fn translate_schemas(serialized_schema: Vec<Vec<u8>>) -> Result<Vec<Schema>, AvroError> {
	let schemas: Vec<Schema> = serialized_schema
		.par_iter()
		.map(|o| -> Schema {
			let schema = translate_schema(o.to_vec());
			match schema {
				Ok(schema) => schema,
				Err(_error) => Schema::Null,
			}
		})
		.collect();

	Ok(schemas)
}

/// Function to get the schema writer with default container as Vec<u8>
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
pub fn get_schema_data_writer<'a>(schema: &'a Schema) -> Writer<'a, Vec<u8>> {
	Writer::with_codec(schema, Vec::new(), Codec::Snappy)
}

/// Function to populate a given schema with data and return serialized record.
/// # Arguments
/// * `schema` - Avro schema
/// * `record` - record to be written as HashMap<String, SchemaValue> where SchemaValue is common types
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
	let mut record_list = Record::new(writer.schema()).unwrap();
	for (field_name, field_value) in records.iter() {
		record_list.put(field_name, field_value.clone());
	}
	let datum_res = to_avro_datum(&schema, record_list)?;
	Ok(datum_res)
}

/// Function to get serialized datum data for a given schema into hashmap.
/// # Arguments
/// *serialized_data* - serialized data to be converted into hashmap
/// * `schema` - Avro schema
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
