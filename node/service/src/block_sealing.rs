use crate::service::new_partial;
use cli_opt::SealingMode;
use common_primitives::node::Hash;
pub use futures::stream::StreamExt;
use sc_service::{Configuration, TaskManager};
use sp_blockchain::HeaderBackend;
use std::{sync::Arc, task::Poll};

// Cumulus
use cumulus_client_consensus_common::ParachainBlockImport;
use cumulus_client_service::prepare_node_config;
use cumulus_primitives_parachain_inherent::MockValidationDataInherentDataProvider;

/// Function to start frequency parachain with instant sealing in dev mode.
/// This function is called when --chain dev --sealing=instant is passed.
#[allow(clippy::expect_used)]
pub fn frequency_dev_sealing(
	config: Configuration,
	sealing_mode: SealingMode,
	sealing_interval: u16,
	sealing_allow_empty_blocks: bool,
) -> Result<TaskManager, sc_service::error::Error> {
	let extra;
	if sealing_mode == SealingMode::Interval {
		extra = format!(" ({}s interval)", sealing_interval);
	} else {
		extra = String::from("");
	}
	log::info!("📎 Development mode (no relay chain) with {} sealing{}", sealing_mode, extra);

	let parachain_config = prepare_node_config(config);

	let sc_service::PartialComponents {
		client,
		backend,
		mut task_manager,
		import_queue,
		keystore_container,
		select_chain: maybe_select_chain,
		transaction_pool,
		other: (_block_import, mut telemetry, _),
	} = new_partial(&parachain_config, true)?;

	// Build the network components required for the blockchain.
	let (network, system_rpc_tx, tx_handler_controller, network_starter) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &parachain_config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			block_announce_validator_builder: None,
			warp_sync: None,
		})?;

	// Start off-chain workers if enabled
	if parachain_config.offchain_worker.enabled {
		let offchain_workers = Arc::new(sc_offchain::OffchainWorkers::new_with_options(
			client.clone(),
			sc_offchain::OffchainWorkerOptions { enable_http_requests: false },
		));

		// Spawn a task to handle off-chain notifications.
		// This task is responsible for processing off-chain events or data for the blockchain.
		task_manager.spawn_handle().spawn(
			"offchain-notifications",
			None,
			sc_offchain::notification_future(
				parachain_config.role.is_authority(),
				client.clone(),
				offchain_workers,
				task_manager.spawn_handle(),
				network.clone(),
			),
		);
	}

	let prometheus_registry = parachain_config.prometheus_registry().cloned();

	let role = parachain_config.role.clone();

	let select_chain = maybe_select_chain
		.expect("In frequency dev mode, `new_partial` will return some `select_chain`; qed");

	// Only block authoring nodes create, seal and finalize blocks
	let command_sink = if role.is_authority() {
		let proposer_factory = sc_basic_authorship::ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool.clone(),
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|x| x.handle()),
		);

		// Channel for the RPC handler to communicate with the authorship task.
		let (command_sink, commands_stream) = futures::channel::mpsc::channel(1024);

		let pool = transaction_pool.pool().clone();

		// For instant sealing, set up a stream that automatically creates and finalizes
		// blocks as soon as transactions arrive.
		// For manual sealing, there's an empty stream.
		// For interval sealing, set up a timed poll.

		let import_stream = match sealing_mode {
			SealingMode::Manual => futures::stream::empty().boxed(),
			SealingMode::Instant =>
				Box::pin(pool.validated_pool().import_notification_stream().map(|_| {
					sc_consensus_manual_seal::rpc::EngineCommand::SealNewBlock {
						create_empty: true,
						finalize: true,
						parent_hash: None,
						sender: None,
					}
				})),
			SealingMode::Interval => {
				let interval = std::time::Duration::from_secs(sealing_interval.into());
				let mut interval_stream = tokio::time::interval(interval);

				Box::pin(futures::stream::poll_fn(move |cx| match interval_stream.poll_tick(cx) {
					Poll::Ready(_instant) => Poll::Ready(Some(
						sc_consensus_manual_seal::rpc::EngineCommand::SealNewBlock {
							create_empty: sealing_allow_empty_blocks,
							finalize: true,
							parent_hash: None,
							sender: None,
						},
					)),
					Poll::Pending => Poll::Pending,
				}))
			},
		};

		let client_for_cidp = client.clone();
		let block_import = ParachainBlockImport::new(client.clone(), backend.clone());

		// Prepare the future for manual sealing block authoring
		let authorship_future =
			sc_consensus_manual_seal::run_manual_seal(sc_consensus_manual_seal::ManualSealParams {
				block_import,
				env: proposer_factory,
				client: client.clone(),
				pool: transaction_pool.clone(),
				commands_stream: futures::stream_select!(commands_stream, import_stream),
				select_chain,
				consensus_data_provider: None,
				create_inherent_data_providers: move |block: Hash, _| {
					let current_para_block = client_for_cidp
						.number(block)
						.expect("Header lookup should succeed")
						.expect("Header passed in as parent should be present in backend.");
					async move {
						let mocked_parachain = MockValidationDataInherentDataProvider {
							current_para_block,
							para_blocks_per_relay_epoch: 0,
							relay_offset: 1000,
							relay_blocks_per_para_block: 2,
							relay_randomness_config: (),
							xcm_config: Default::default(),
							raw_downward_messages: vec![],
							raw_horizontal_messages: vec![],
						};
						Ok((
							sp_timestamp::InherentDataProvider::from_system_time(),
							mocked_parachain,
						))
					}
				},
			});

		// Spawn a background task for block authoring
		task_manager.spawn_essential_handle().spawn_blocking(
			match sealing_mode {
				SealingMode::Manual => "manual-seal",
				SealingMode::Instant => "instant-seal",
				SealingMode::Interval => "interval-seal",
			},
			Some("block-authoring"),
			authorship_future,
		);
		Some(command_sink)
	} else {
		None
	};

	// Build the RPC
	let rpc_extensions_builder = {
		let client = client.clone();
		let transaction_pool = transaction_pool.clone();

		move |deny_unsafe, _| {
			let deps = crate::rpc::FullDeps {
				client: client.clone(),
				pool: transaction_pool.clone(),
				deny_unsafe,
				command_sink: command_sink.clone(),
			};

			crate::rpc::create_full(deps).map_err(Into::into)
		}
	};

	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		rpc_builder: Box::new(rpc_extensions_builder),
		client: client.clone(),
		transaction_pool: transaction_pool.clone(),
		task_manager: &mut task_manager,
		config: parachain_config,
		keystore: keystore_container.sync_keystore(),
		backend,
		network: network.clone(),
		system_rpc_tx,
		tx_handler_controller,
		telemetry: telemetry.as_mut(),
	})?;

	network_starter.start_network();

	Ok(task_manager)
}
