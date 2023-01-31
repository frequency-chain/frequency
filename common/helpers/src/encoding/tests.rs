use crate::{
	avro,
	encoding::{
		avro_binary::AvroBinaryEncoding, message_pack::MessagePackEncoding,
		protocol_buf::ProtocolBufEncoding, thrift::ThriftEncoding, traits::Encoding,
	},
	types::SchemaValue,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestMessage {
	data: Vec<u8>,
}

impl TestMessage {
	fn new(size: usize) -> Self {
		let mut data = vec![];
		for _ in 0..size {
			data.push(0);
		}
		Self { data }
	}
}

impl Into<thrift_codec::data::Struct> for TestMessage {
	fn into(self) -> thrift_codec::data::Struct {
		thrift_codec::data::Struct::from(("data", self.data))
	}
}

fn print_metrics(metrics: &crate::encoding::traits::EncodingMetrics) {
	println!("Encoded size: {}", metrics.encoded_size);
	println!("Decoding time: {}", metrics.decoding_time);
	println!("Compression ratio: {}", metrics.compression_ratio);
	println!("Encoding time: {}", metrics.encoding_time);
}

#[test]
fn protobuf_encoding_base_test() {
	use protobuf::Message;
	let encoder = ProtocolBufEncoding::new();
	let data = protobuf::well_known_types::timestamp::Timestamp::now();
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

#[test]
fn test_thrift_encoding_size() {
	let thrift_encoding = ThriftEncoding::new();
	let mut results = vec![];

	for size in [5_000, 10_000, 20_000, 32_000, 64_000].iter() {
		let test_message = TestMessage::new(*size);
		let message = thrift_codec::message::Message::oneway("test_method", 1, test_message.into());
		thrift_encoding.encode(&message);
		let input_size = *size;
		let metrics = thrift_encoding.get_metrics(&message, input_size);
		results.push((input_size, metrics));
	}
	for (size, metrics) in results {
		println!("Data size: {:6} bytes", size);
		print_metrics(&metrics);
	}
}

#[test]
fn test_message_pack_encoding() {
	let sizes = [5_000, 10_000, 20_000, 40_000, 64_000];
	let mut results = vec![];

	for &size in sizes.iter() {
		let test_message = TestMessage::new(size);
		let message_pack = MessagePackEncoding::new();

		let metrics = message_pack.get_metrics(&test_message, size);
		results.push((size, metrics));
	}

	for (size, metrics) in results {
		println!("Data size: {:6} bytes", size);
		print_metrics(&metrics);
	}
}
