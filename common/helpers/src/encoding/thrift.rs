use crate::encoding::traits::{Encoding, EncodingMetrics};
use std::{io::Cursor, time::Instant};
use thrift_codec::{message::Message, CompactDecode, CompactEncode};

pub struct ThriftEncoding;

impl ThriftEncoding {
	pub fn new() -> Self {
		Self {}
	}
}

impl<Message: CompactEncode + CompactDecode> Encoding<Message> for ThriftEncoding {
	fn encode(&self, data: &Message) -> Vec<u8> {
		let mut encoded = Vec::new();
		data.compact_encode(&mut encoded).unwrap();
		encoded
	}

	fn decode(&self, data: &[u8]) -> Message {
		let message = Message::compact_decode(&mut &data[..]).unwrap();
		message
	}

	fn get_metrics(&self, data: &Message, input_size: usize) -> EncodingMetrics {
		let start_encode = Instant::now();
		let encoded = self.encode(data);
		let encoding_time = start_encode.elapsed().as_secs_f64();

		let encoded_size = encoded.len();
		let compression_ratio = (encoded_size as f64) / (input_size as f64);

		let start_decode = Instant::now();
		<ThriftEncoding as Encoding<Message>>::decode(self, &encoded);
		let decoding_time = start_decode.elapsed().as_secs_f64();

		EncodingMetrics { encoded_size, encoding_time, decoding_time, compression_ratio }
	}
}
