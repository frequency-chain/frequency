use crate::{
	encoding::traits::{Encoding, EncodingMetrics},
	types::SchemaValue,
};
use apache_avro::{from_avro_datum, schema::Schema, to_avro_datum, types::Record, Codec, Writer};
use std::{collections::HashMap, io::Cursor, time::Instant};

pub struct AvroBinaryEncoding {
	schema: Schema,
	codec: Codec,
}

pub type AvroHashMap = HashMap<String, SchemaValue>;

impl AvroBinaryEncoding {
	pub fn new(schema: Schema, codec: Codec) -> Self {
		Self { schema, codec }
	}
}

impl Encoding<AvroHashMap> for AvroBinaryEncoding {
	fn encode(&self, data: &AvroHashMap) -> Vec<u8> {
		let writer = Writer::with_codec(&self.schema, Vec::new(), self.codec);
		match Record::new(writer.schema()) {
			None => return to_avro_datum(&self.schema, SchemaValue::Null).unwrap_or_default(),
			Some(mut record_list) => {
				for (field_name, field_value) in data.iter() {
					record_list.put(field_name, field_value.clone());
				}
				return to_avro_datum(&self.schema, record_list).unwrap_or_default()
			},
		}
	}

	fn decode(&self, data: &[u8]) -> AvroHashMap {
		let from_data_datum = from_avro_datum(&self.schema, &mut Cursor::new(data), None)
			.unwrap_or(SchemaValue::Null);
		let mut result_record = HashMap::<String, SchemaValue>::new();

		match from_data_datum {
			SchemaValue::Record(record) =>
				for (field_name, field_value) in record.iter() {
					result_record.insert(field_name.clone(), field_value.clone());
				},
			_ => {},
		}
		result_record
	}

	fn get_metrics(&self, data: &AvroHashMap, input_size: usize) -> EncodingMetrics {
		let start_encode = Instant::now();
		let encoded = self.encode(data);
		let encoding_time = start_encode.elapsed().as_secs_f64();

		let encoded_size = encoded.len();
		let compression_ratio = (input_size as f64) / (encoded_size as f64);

		let start_decode = Instant::now();
		<AvroBinaryEncoding as Encoding<AvroHashMap>>::decode(self, &encoded);
		let decoding_time = start_decode.elapsed().as_secs_f64();

		EncodingMetrics { encoded_size, decoding_time, compression_ratio, encoding_time }
	}
}
