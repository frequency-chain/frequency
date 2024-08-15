#![allow(missing_docs)]
use std::net::SocketAddr;

const HTTP_PREFIX: &str = "http://";

/// Normalize and convert SocketAddr to string
pub fn convert_address_to_normalized_string(addr: &Option<SocketAddr>) -> Option<Vec<u8>> {
	let mut address = match addr {
		None => return None,
		Some(SocketAddr::V4(v4)) => v4.to_string(),
		Some(SocketAddr::V6(v6)) => v6.to_string(),
	};

	if !address.starts_with(HTTP_PREFIX) {
		address = format!("{}{}", HTTP_PREFIX, address);
	}
	Some(address.into_bytes())
}
