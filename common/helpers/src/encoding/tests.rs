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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TestMessage {
	data: Vec<u8>,
}

impl TestMessage {
	fn new(data: Vec<u8>) -> Self {
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
fn protobuf_encoding_test() {
	use protobuf::Message;
	let encoder = ProtocolBufEncoding::new();
	let sizes = [5_000, 10_000, 20_000, 40_000, 64_000];
	let mut results = vec![];
	
	for &size in sizes.iter() {
		let data: Vec<u8> = (0..size).map(|_| rand::random::<u8>()).collect();
		let test_message: protobuf::well_known_types::wrappers::BytesValue = data.into();
		let encoded = encoder.encode(&test_message);
		let encoded_size = encoded.len();
		let metrics = encoder.get_metrics(&test_message, test_message.compute_size() as usize);
		assert_eq!(metrics.encoded_size, encoded_size);
		results.push((size, metrics));
	}
	for (size, metrics) in results {
		println!("Data size: {:6} bytes", size);
		print_metrics(&metrics);
	}
}

#[test]
fn avro_encoding_base_test() {
	let raw_schema = r#"
    {
        "type": "record",
        "name": "test",
        "fields": [
            {"name": "data", "type": "bytes"}
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
	let sizes = [5_000, 10_000, 20_000, 40_000, 64_000];
	let mut results = vec![];

	// using snappy compression
	let codec = apache_avro::Codec::Snappy;
	let encoder = AvroBinaryEncoding::new(translated_schema, codec);
	for &size in sizes.iter() {
		let data: Vec<u8> = (0..size).map(|_| rand::random::<u8>()).collect();
		data_map.insert("data".to_string(), SchemaValue::Bytes(data));
		let encoded = encoder.encode(&data_map);
		let encoded_size = encoded.len();
		let metrics = encoder.get_metrics(&data_map, size);
		assert_eq!(metrics.encoded_size, encoded_size);
		results.push((size, metrics));
	}
	for (size, metrics) in results {
		println!("Data size: {:6} bytes", size);
		print_metrics(&metrics);
	}
}

#[test]
fn test_thrift_encoding_size() {
	let thrift_encoding = ThriftEncoding::new();
	let sizes = [5_000, 10_000, 20_000, 40_000, 64_000];
	let mut results = vec![];

	for &size in sizes.iter() {
		let data: Vec<u8> = (0..size).map(|_| rand::random::<u8>()).collect();
		let test_message = TestMessage::new(data);
		let message = thrift_codec::message::Message::oneway("test_method", 1, test_message.into());
		thrift_encoding.encode(&message);
		let metrics = thrift_encoding.get_metrics(&message, size);
		results.push((size, metrics));
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
		let data: Vec<u8> = (0..size).map(|_| rand::random::<u8>()).collect();
		let test_message = TestMessage::new(data);
		let message_pack = MessagePackEncoding::new();

		let metrics = message_pack.get_metrics(&test_message, size);
		results.push((size, metrics));
	}

	for (size, metrics) in results {
		println!("Data size: {:6} bytes", size);
		print_metrics(&metrics);
	}
}
