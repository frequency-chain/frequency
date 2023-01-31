use crate::encoding::traits::{Encoding, EncodingMetrics};
use std::time::Instant;
use thrift_codec::{CompactDecode, CompactEncode};

pub struct ThriftEncoding;

impl ThriftEncoding {
	pub fn new() -> Self {
		Self {}
	}
}

impl<T: CompactEncode + CompactDecode> Encoding<T> for ThriftEncoding {
	fn encode(&self, data: &T) -> Vec<u8> {
		let mut encoded = Vec::new();
		data.compact_encode(&mut encoded).unwrap();
		encoded
	}

	fn decode(&self, data: &[u8]) -> T {
		let message = T::compact_decode(&mut &data[..]).unwrap();
		message
	}

	fn get_metrics(&self, data: &T, input_size: usize) -> EncodingMetrics {
		let start_encode = Instant::now();
		let encoded = self.encode(data);
		let encoding_time = start_encode.elapsed().as_secs_f64();

		let encoded_size = encoded.len();
		let compression_ratio = (encoded_size as f64) / (input_size as f64);

		let start_decode = Instant::now();
		<ThriftEncoding as Encoding<T>>::decode(self, &encoded);
		let decoding_time = start_decode.elapsed().as_secs_f64();

		EncodingMetrics { encoded_size, encoding_time, decoding_time, compression_ratio }
	}
}
