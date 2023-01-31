use crate::{
	avro,
	encoding::{
		avro_binary::AvroBinaryEncoding, protocol_buf::ProtocolBufEncoding, thrift::ThriftEncoding,
		traits::Encoding,
	},
	types::SchemaValue,
};
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
	let codec = apache_avro::Codec::Snappy;
	let encoder = AvroBinaryEncoding::new(translated_schema, codec);
	let encoded = encoder.encode(&data_map);
	let encoded_size = encoded.len();
	let metrics = encoder.get_metrics(&data_map, data_map.len());
	assert_eq!(metrics.encoded_size, encoded_size);
	print_metrics(&metrics);
}

use thrift_codec::data::{Field, Struct};

#[derive(Debug, PartialEq, Struct)]
struct TestMessage {
	#[thrift_field(1)]
	field1: String,
}

#[test]
fn test_thrift_encoding_size() {
	let thrift_encoding = ThriftEncoding::new();

	for size in [5_000, 10_000, 20_000, 32_000, 64_000].iter() {
		let message = TestMessage { field1: "a".repeat(*size) };
		let data = thrift_encoding.encode(&message);
		let input_size = *size;
		let metrics = thrift_encoding.get_metrics(&data, input_size);
		print_metrics(&metrics);
	}
}
