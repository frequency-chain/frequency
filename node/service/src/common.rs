#![allow(missing_docs)]
use sc_service::config::RpcEndpoint;
use std::net::SocketAddr;

const HTTP_PREFIX: &str = "http://";

/// Normalize and convert SocketAddr to string
pub fn listen_addrs_to_normalized_strings(addr: &Option<Vec<RpcEndpoint>>) -> Option<Vec<Vec<u8>>> {
	match addr {
		None => None,
		Some(rpc_endpoints) => {
			let endpoints = rpc_endpoints
				.iter()
				.map(|endpoint| {
					let socket_addr = endpoint.listen_addr;
					let mut address = match socket_addr {
						SocketAddr::V4(v4) => v4.to_string(),
						SocketAddr::V6(v6) => v6.to_string(),
						_ => "".to_string(),
					};
					if !address.starts_with(HTTP_PREFIX) {
						address = format!("{}{}", HTTP_PREFIX, address);
					}
					address.into_bytes()
				})
				.filter(|addr| addr.len() > 0)
				.collect();
			Some(endpoints)
		},
	}
}
