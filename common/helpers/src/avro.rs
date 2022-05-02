use crate::types::*;
use apache_avro::{schema::Schema, types::Record, Codec, Reader, Writer};
use std::{
	collections::HashMap,
	io::{Read, Write},
	str,
};

#[derive(thiserror::Error, Debug)]
pub enum AvroError {
	#[error("I/O error")]
	Io(#[from] std::io::Error),
	#[error("Avro error")]
	Avro(#[from] apache_avro::Error),
}

pub fn set_writer_schema<'a, W: Write>(writer: W, schema: &'a Schema) -> Writer<'a, W> {
	Writer::with_codec(schema, writer, Codec::Snappy)
}

pub fn get_writer_schema<'a>(schema: &'a Schema) -> Writer<'a, Vec<u8>> {
	Writer::with_codec(schema, Vec::new(), Codec::Snappy)
}

pub fn populate_record(
	writer: &mut Writer<Vec<u8>>,
	record: &HashMap<String, SchemaValue>,
) -> Result<(), AvroError> {
	let mut record_list = Record::new(writer.schema()).unwrap();
	for (field_name, field_value) in record.iter() {
		record_list.put(field_name, field_value.clone());
	}
	writer.append(record_list)?;
	Ok(())
}

pub fn get_reader_schema<'a, R: Read>(
	reader: R,
	schema: &'a Schema,
) -> Result<Reader<'a, R>, AvroError> {
	Ok(Reader::with_schema(&schema, reader)?)
}

pub fn translate_schema(schema: Vec<u8>) -> Result<Schema, AvroError> {
	let schema_str = str::from_utf8(&schema);
	match schema_str {
		Ok(schema_str) => {
			let schema = Schema::parse_str(schema_str)?;
			Ok(schema)
		},
		Err(error) => Err(AvroError::InvalidSchema(error.to_string())),
	}
}

pub fn translate_schemas(schema: Vec<Vec<u8>>) -> Result<Vec<Schema>, AvroError> {
	let mut schemas = Vec::new();
	for s in schema {
		schemas.push(translate_schema(s)?);
	}
	Ok(schemas)
}

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
