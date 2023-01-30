use crate::encoding::{protocol_buf::ProtocolBufEncoding, traits::Encoding};
use protobuf::{well_known_types::timestamp::Timestamp, Message};

fn print_metrics(metrics: &crate::encoding::traits::EncodingMetrics) {
	println!("Encoded size: {}", metrics.encoded_size);
	println!("Decoding time: {}", metrics.decoding_time);
	println!("Compression ratio: {}", metrics.compression_ratio);
	println!("Encoding time: {}", metrics.encoding_time);
}

#[test]
fn protobuf_encoding_base_test() {
	let data = Timestamp::now();
	let encoded = ProtocolBufEncoding.encode(&data);
	let encoded_size = encoded.len();
	let compression_ratio = (data.compute_size() as f64) / (encoded_size as f64);
	let metrics = ProtocolBufEncoding.get_metrics(&data, data.compute_size() as usize);
	assert_eq!(metrics.encoded_size, encoded_size);
	assert_eq!(metrics.compression_ratio, compression_ratio);
	print_metrics(&metrics);
}
