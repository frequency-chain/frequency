/// metrics for encoding
pub struct EncodingMetrics {
	pub encoded_size: usize,
	pub decoding_time: f64,
	pub compression_ratio: f64,
	pub encoding_time: f64,
}

/// Generic encoding trait
pub trait Encoding<T> {
	fn encode(&self, data: &T) -> Vec<u8>;
	fn decode(&self, data: &[u8]) -> T;
	fn get_metrics(&self, data: &T, input_size: usize) -> EncodingMetrics;
}
