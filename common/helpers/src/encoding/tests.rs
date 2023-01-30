use crate::{
	encoding::{
		avro_binary::AvroBinaryEncoding, protocol_buf::ProtocolBufEncoding, traits::Encoding,
	},
	types::SchemaValue,
};
use apache_avro::Schema;
use protobuf::{well_known_types::timestamp::Timestamp, Message};
use std::collections::HashMap;

fn print_metrics(metrics: &crate::encoding::traits::EncodingMetrics) {
	println!("Encoded size: {}", metrics.encoded_size);
	println!("Decoding time: {}", metrics.decoding_time);
	println!("Compression ratio: {}", metrics.compression_ratio);
	println!("Encoding time: {}", metrics.encoding_time);
}

#[test]
fn protobuf_encoding_base_test() {
	let encoder = ProtocolBufEncoding::new();
	let data = Timestamp::now();
	let encoded = encoder.encode(&data);
	let encoded_size = encoded.len();
	let compression_ratio = (data.compute_size() as f64) / (encoded_size as f64);
	let metrics = encoder.get_metrics(&data, data.compute_size() as usize);
	assert_eq!(metrics.encoded_size, encoded_size);
	assert_eq!(metrics.compression_ratio, compression_ratio);
	print_metrics(&metrics);
}

#[test]
fn avro_encoding_base_test() {
	let schema = r#"
    {
        "type": "record",
        "name": "User",
        "fields": [
            {"name": "name", "type": "string"},
            {"name": "favorite_number",  "type": ["int", "null"]},
            {"name": "favorite_color", "type": ["string", "null"]}
        ]
    }
    "#;
	let codec = apache_avro::Codec::Snappy;
	let encoder = AvroBinaryEncoding::new(Schema::parse_str(schema).unwrap(), codec);
	let mut data = HashMap::<String, SchemaValue>::new();
	data.insert("name".to_string(), SchemaValue::String("Avro".to_string()));
	data.insert("favorite_number".to_string(), SchemaValue::Int(256));
	data.insert("favorite_color".to_string(), SchemaValue::String("spectrum".to_string()));
	let encoded = encoder.encode(&data);
	let encoded_size = encoded.len();
	let metrics = encoder.get_metrics(&data, data.len());
	assert_eq!(metrics.encoded_size, encoded_size);
	print_metrics(&metrics);
}
