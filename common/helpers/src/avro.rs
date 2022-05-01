use apache_avro::schema::Schema;

#[derive(thiserror::Error, Debug)]
pub enum AvroError {
	#[error("I/O error")]
	Io(#[from] std::io::Error),
	#[error("Avro error")]
	Avro(#[from] apache_avro::Error),
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
