use crate::encoding::traits::{Encoding, EncodingMetrics};
use rmp_serde::{Deserializer, Serializer};
use std::{io::Cursor, time::Instant};

pub struct MessagePackEncoding;

impl MessagePackEncoding {
	pub fn new() -> Self {
		Self {}
	}
}

impl<T> Encoding<T> for MessagePackEncoding
where
	T: serde::Serialize + for<'a> serde::Deserialize<'a>,
{
	fn encode(&self, data: &T) -> Vec<u8> {
		let mut buf = Vec::new();
		let mut serializer = Serializer::new(&mut buf);
		data.serialize(&mut serializer).unwrap();
		buf
	}

	fn decode(&self, data: &[u8]) -> T {
		let mut deserializer = Deserializer::new(Cursor::new(data));
		T::deserialize(&mut deserializer).unwrap()
	}

	fn get_metrics(&self, data: &T, input_size: usize) -> EncodingMetrics {
		let start_encode = Instant::now();
		let encoded = self.encode(data);
		let encoding_time = start_encode.elapsed().as_secs_f64();

		let encoded_size = encoded.len();
		let compression_ratio = (input_size as f64) / (encoded_size as f64);

		let start_decode = Instant::now();
		<MessagePackEncoding as Encoding<T>>::decode(self, &encoded);
		let decoding_time = start_decode.elapsed().as_secs_f64();

		EncodingMetrics { encoded_size, encoding_time, decoding_time, compression_ratio }
	}
}
