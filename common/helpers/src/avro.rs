use apache_avro::{schema::Schema, Codec, Reader, Writer, types::Record};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::str;
use crate::types::*;

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

pub fn populate_record<W: Write>(
	writer: &mut Writer<W>,
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
	let schema_str = str::from_utf8(&schema)?;
	Ok(Schema::parse_str(schema_str)?)
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
	let schema = Schema::parse_str(raw_schema)?;
	// get the schema finger print
	let schema_canonical_form = schema.canonical_form();
	Ok((schema, schema_canonical_form.as_bytes().to_vec()))
}

pub fn fingerprint_raw_schema_list(
	raw_schema: &[&str],
) -> Result<(Vec<Schema>, Vec<Vec<u8>>), AvroError> {
	let mut schema_fingerprints = Vec::new();
	let mut schemas = Vec::new();
	// iterate over schema list and generate schema fingerprints
	for raw_schema in raw_schema {
		// parse_str will fail if schema is not valid
		let schema = Schema::parse_str(raw_schema)?;
		// get the schema finger print
		let schema_canonical_form = schema.canonical_form();
		// add to return
		schema_fingerprints.push(schema_canonical_form.as_bytes().to_vec());
		schemas.push(schema);
	}
	Ok((schemas, schema_fingerprints))
}
