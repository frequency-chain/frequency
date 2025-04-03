#![allow(missing_docs)]
use crate::service::ParachainClient;
use common_primitives::{node::Block, offchain::OcwCustomExt};
use sc_client_api::Backend;
use sc_offchain::NetworkProvider;
use sc_service::{config::RpcEndpoint, TFullBackend, TaskManager};
use sc_transaction_pool_api::OffchainTransactionPoolFactory;
use sp_core::offchain::OffchainStorage;
use sp_keystore::KeystorePtr;
use std::{net::SocketAddr, sync::Arc};

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
type ParachainBackend = TFullBackend<Block>;
pub fn start_offchain_workers(
	client: &Arc<ParachainClient>,
	parachain_config: &sc_service::Configuration,
	keystore: Option<KeystorePtr>,
	backend: &Arc<ParachainBackend>,
	transaction_pool: Option<OffchainTransactionPoolFactory<Block>>,
	network_provider: Arc<dyn NetworkProvider + Send + Sync>,
	task_manager: &TaskManager,
) {
	use futures::FutureExt;
	let rpc_addresses = listen_addrs_to_normalized_strings(&parachain_config.rpc.addr)
		.expect("config.rpc.addr has invalid input");
	let is_validator = parachain_config.role.is_authority();
	let offchain_workers = sc_offchain::OffchainWorkers::new(sc_offchain::OffchainWorkerOptions {
		runtime_api_provider: client.clone(),
		is_validator,
		keystore,
		offchain_db: backend.offchain_storage(),
		transaction_pool,
		network_provider,
		enable_http_requests: true,
		custom_extensions: move |_hash| {
			let rpc_address = rpc_addresses.get(0).expect("no rpc addresses in config");
			vec![Box::new(OcwCustomExt(rpc_address.clone())) as Box<_>]
		},
	})
	.expect("Could not create Offchain Worker");

	// Spawn a task to handle off-chain notifications.
	// This task is responsible for processing off-chain events or data for the blockchain.
	task_manager.spawn_handle().spawn(
		"offchain-workers-runner",
		"offchain-work",
		offchain_workers.run(client.clone(), task_manager.spawn_handle()).boxed(),
	);
}
