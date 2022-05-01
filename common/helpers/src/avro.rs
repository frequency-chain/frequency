use apache_avro::schema::Schema;
use sha2::Sha256;

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
	let schema_fingerprint = schema.fingerprint::<Sha256>();
	Ok((schema, schema_fingerprint.bytes))
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
		let schema_fingerprint = schema.fingerprint::<Sha256>();
		// add to return
		schema_fingerprints.push(schema_fingerprint.bytes);
		schemas.push(schema);
	}
	Ok((schemas, schema_fingerprints))
}
