use crate::{eth::new_frontier_partial, service::new_partial};
use cli_opt::SealingMode;
pub use futures::stream::StreamExt;
use sc_consensus::block_import::BlockImport;

use crate::{
	common::start_offchain_workers,
	eth::{spawn_frontier_tasks, FrontierPartialComponents},
	service::HostFunctions,
};
use cli_opt::EthConfiguration;
use common_primitives::node::{Block, Hash};
use core::marker::PhantomData;
use frequency_runtime::TransactionConverter;
use futures::Stream;
use sc_client_api::backend::{Backend as ClientBackend, Finalizer};
use sc_consensus_manual_seal::{
	finalize_block, EngineCommand, FinalizeBlockParams, ManualSealParams, MANUAL_SEAL_ENGINE_ID,
};
use sc_network::NetworkBackend;
use sc_service::{Configuration, TaskManager};
use sc_transaction_pool::TransactionPoolOptions;
use sc_transaction_pool_api::{OffchainTransactionPoolFactory, TransactionPool};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_consensus::{Environment, Proposer, SelectChain};
use sp_core::{H256, U256};
use sp_inherents::CreateInherentDataProviders;
use sp_runtime::traits::Block as BlockT;
use std::{sync::Arc, task::Poll};

/// Function to start Frequency in dev mode without a relay chain
/// This function is called when --chain dev --sealing= is passed.
#[allow(clippy::expect_used)]
pub async fn start_frequency_dev_sealing_node(
	mut config: Configuration,
	eth_config: EthConfiguration,
	sealing_mode: SealingMode,
	sealing_interval: u16,
	sealing_create_empty_blocks: bool,
	override_pool_config: Option<TransactionPoolOptions>,
) -> Result<TaskManager, sc_service::error::Error> {
	let extra: String = if sealing_mode == SealingMode::Interval {
		format!(" ({}s interval)", sealing_interval)
	} else {
		String::from("")
	};
	log::info!("📎 Development mode (no relay chain) with {} sealing{}", sealing_mode, extra);

	let net_config = sc_network::config::FullNetworkConfiguration::<
		_,
		_,
		sc_network::NetworkWorker<Block, Hash>,
	>::new(&config.network, None); // 2nd param, metrics_registry: Option<Metrics>
	let sc_service::PartialComponents {
		client,
		backend,
		mut task_manager,
		import_queue,
		keystore_container,
		select_chain: maybe_select_chain,
		transaction_pool,
		other: (_block_import, mut telemetry, _, frontier_backend, storage_override),
	} = new_partial(&config, &eth_config, true, override_pool_config)?;
	let metrics = sc_network::NetworkWorker::<Block, Hash>::register_notification_metrics(
		config.prometheus_config.as_ref().map(|cfg| &cfg.registry),
	);

	let FrontierPartialComponents { filter_pool, fee_history_cache, fee_history_cache_limit } =
		new_frontier_partial(&eth_config)?;

	// Build the network components required for the blockchain.
	let (network, system_rpc_tx, tx_handler_controller, sync_service) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			net_config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			block_announce_validator_builder: None,
			warp_sync_config: None,
			block_relay: None,
			metrics,
		})?;

	// Start off-chain workers if enabled
	if config.offchain_worker.enabled {
		log::info!("OFFCHAIN WORKER is Enabled!");
		start_offchain_workers(
			&client,
			&config,
			Some(keystore_container.keystore()),
			&backend,
			Some(OffchainTransactionPoolFactory::new(transaction_pool.clone())),
			Arc::new(network.clone()),
			&task_manager,
		);
	}

	let prometheus_registry = config.prometheus_registry().cloned();

	let role = config.role;

	let frontier_backend = Arc::new(frontier_backend);

	let select_chain = maybe_select_chain
		.expect("In frequency dev mode, `new_partial` will return some `select_chain`; qed");

	// Sinks for pubsub notifications.
	// Everytime a new subscription is created, a new mpsc channel is added to the sink pool.
	// The MappingSyncWorker sends through the channel on block import and the subscription emits a notification to the subscriber on receiving a message through this channel.
	// This way we avoid race conditions when using native substrate block import notification stream.
	let pubsub_notification_sinks: fc_mapping_sync::EthereumBlockNotificationSinks<
		fc_mapping_sync::EthereumBlockNotification<Block>,
	> = Default::default();
	let pubsub_notification_sinks = Arc::new(pubsub_notification_sinks);

	// for ethereum-compatibility rpc.
	config.rpc.id_provider = Some(Box::new(fc_rpc::EthereumSubIdProvider));

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

		let pool = transaction_pool.clone();

		// For instant sealing, set up a stream that automatically creates and finalizes
		// blocks as soon as transactions arrive.
		// For manual sealing, there's an empty stream.
		// For interval sealing, set up a timed poll.

		let import_stream = match sealing_mode {
			SealingMode::Manual => futures::stream::empty().boxed(),
			SealingMode::Instant =>
				Box::pin(pool.import_notification_stream().map(|_| EngineCommand::SealNewBlock {
					create_empty: true,
					finalize: true,
					parent_hash: None,
					sender: None,
				})),
			SealingMode::Interval => {
				let interval = std::time::Duration::from_secs(sealing_interval.into());
				let mut interval_stream = tokio::time::interval(interval);

				Box::pin(futures::stream::poll_fn(move |cx| {
					let engine_seal_cmd = EngineCommand::SealNewBlock {
						create_empty: sealing_create_empty_blocks,
						finalize: true,
						parent_hash: None,
						sender: None,
					};
					match interval_stream.poll_tick(cx) {
						Poll::Ready(_instant) => {
							log::info!("⏳ Interval timer triggered");
							Poll::Ready(Some(engine_seal_cmd))
						},
						Poll::Pending => Poll::Pending,
					}
				}))
			},
		};

		let slot_duration = sc_consensus_aura::slot_duration(&*client)?;
		let target_gas_price = eth_config.target_gas_price;
		let create_inherent_data_providers = move |_, ()| async move {
			let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
			let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
				*timestamp,
				slot_duration,
			);
			let dynamic_fee = fp_dynamic_fee::InherentDataProvider(U256::from(target_gas_price));
			Ok((slot, timestamp, dynamic_fee))
		};

		// Prepare the future for manual sealing block authoring
		let authorship_future = run_seal_command(sc_consensus_manual_seal::ManualSealParams {
			block_import: client.clone(),
			env: proposer_factory,
			client: client.clone(),
			pool: transaction_pool.clone(),
			commands_stream: futures::stream_select!(commands_stream, import_stream),
			select_chain,
			consensus_data_provider: None,
			create_inherent_data_providers,
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
		let backend = match config.offchain_worker.enabled {
			true => backend.offchain_storage(),
			false => None,
		};
		let network = network.clone();
		let sync_service = sync_service.clone();

		let is_authority = role.is_authority();
		let enable_dev_signer = eth_config.enable_dev_signer;
		let max_past_logs = eth_config.max_past_logs;
		let execute_gas_limit_multiplier = eth_config.execute_gas_limit_multiplier;
		let filter_pool = filter_pool.clone();
		let frontier_backend = frontier_backend.clone();
		let pubsub_notification_sinks = pubsub_notification_sinks.clone();
		let storage_override = storage_override.clone();
		let fee_history_cache = fee_history_cache.clone();
		let block_data_cache = Arc::new(fc_rpc::EthBlockDataCacheTask::new(
			task_manager.spawn_handle(),
			storage_override.clone(),
			eth_config.eth_log_block_cache,
			eth_config.eth_statuses_cache,
			prometheus_registry.clone(),
		));

		let slot_duration = sc_consensus_aura::slot_duration(&*client)?;
		let target_gas_price = eth_config.target_gas_price;
		let pending_create_inherent_data_providers = move |_, ()| async move {
			let current = sp_timestamp::InherentDataProvider::from_system_time();
			let next_slot = current.timestamp().as_millis() + slot_duration.as_millis();
			let timestamp = sp_timestamp::InherentDataProvider::new(next_slot.into());
			let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
				*timestamp,
				slot_duration,
			);
			let dynamic_fee = fp_dynamic_fee::InherentDataProvider(U256::from(target_gas_price));
			Ok((slot, timestamp, dynamic_fee))
		};

		Box::new(move |subscription_task_executor| {
			let eth_deps = crate::rpc::EthDeps {
				client: client.clone(),
				pool: transaction_pool.clone(),
				graph: transaction_pool.clone(),
				converter: Some(TransactionConverter::<Block>::default()),
				is_authority,
				enable_dev_signer,
				network: network.clone(),
				sync: sync_service.clone(),
				frontier_backend: match &*frontier_backend {
					fc_db::Backend::KeyValue(b) => b.clone(),
					fc_db::Backend::Sql(b) => b.clone(),
				},
				storage_override: storage_override.clone(),
				block_data_cache: block_data_cache.clone(),
				filter_pool: filter_pool.clone(),
				max_past_logs,
				fee_history_cache: fee_history_cache.clone(),
				fee_history_cache_limit,
				execute_gas_limit_multiplier,
				forced_parent_hashes: None,
				pending_create_inherent_data_providers,
			};
			let deps = crate::rpc::FullDeps {
				client: client.clone(),
				pool: transaction_pool.clone(),
				command_sink: command_sink.clone(),
				eth: eth_deps,
			};
			crate::rpc::create_full::<_, _, _, sc_client_db::Backend<Block>, _, _, _>(
				deps,
				backend.clone(),
				subscription_task_executor,
				pubsub_notification_sinks.clone(),
			)
			.map_err(Into::into)
		})
	};

	let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		rpc_builder: rpc_extensions_builder,
		client: client.clone(),
		transaction_pool: transaction_pool.clone(),
		task_manager: &mut task_manager,
		config,
		keystore: keystore_container.keystore(),
		backend: backend.clone(),
		network: network.clone(),
		sync_service: sync_service.clone(),
		system_rpc_tx,
		tx_handler_controller,
		telemetry: telemetry.as_mut(),
	})?;

	spawn_frontier_tasks::<_, frequency_runtime::RuntimeApi, HostFunctions>(
		&task_manager,
		client.clone(),
		backend,
		frontier_backend,
		filter_pool,
		storage_override,
		fee_history_cache,
		fee_history_cache_limit,
		sync_service.clone(),
		pubsub_notification_sinks,
	)
	.await;

	Ok(task_manager)
}

/// Override manual sealing to handle creating non-empty blocks for interval sealing mode without nuisance warning logs
async fn run_seal_command<B, BI, CB, E, C, TP, SC, CS, CIDP, P>(
	ManualSealParams {
		mut block_import,
		mut env,
		client,
		pool,
		mut commands_stream,
		select_chain,
		consensus_data_provider,
		create_inherent_data_providers,
	}: ManualSealParams<B, BI, E, C, TP, SC, CS, CIDP, P>,
) where
	B: BlockT + 'static,
	BI: BlockImport<B, Error = sp_consensus::Error> + Send + Sync + 'static,
	C: HeaderBackend<B> + Finalizer<B, CB> + ProvideRuntimeApi<B> + 'static,
	CB: ClientBackend<B> + 'static,
	E: Environment<B> + 'static,
	E::Proposer: Proposer<B, Proof = P>,
	CS: Stream<Item = EngineCommand<<B as BlockT>::Hash>> + Unpin + 'static,
	SC: SelectChain<B> + 'static,
	TP: TransactionPool<Block = B>,
	CIDP: CreateInherentDataProviders<B, ()>,
	P: parity_scale_codec::Encode + Send + Sync + 'static,
{
	while let Some(command) = commands_stream.next().await {
		match command {
			EngineCommand::SealNewBlock { create_empty, finalize, parent_hash, sender } =>
			// To keep the logs quieter, don't even attempt to create empty blocks unless there are transactions in the pool
				if create_empty || pool.ready().count() > 0 {
					sc_consensus_manual_seal::seal_block(
						sc_consensus_manual_seal::SealBlockParams {
							sender,
							parent_hash,
							finalize,
							create_empty,
							env: &mut env,
							select_chain: &select_chain,
							block_import: &mut block_import,
							consensus_data_provider: consensus_data_provider.as_deref(),
							pool: pool.clone(),
							client: client.clone(),
							create_inherent_data_providers: &create_inherent_data_providers,
						},
					)
					.await;
				},
			EngineCommand::FinalizeBlock { hash, sender, justification } => {
				let justification = justification.map(|j| (MANUAL_SEAL_ENGINE_ID, j));
				finalize_block(FinalizeBlockParams {
					hash,
					sender,
					justification,
					finalizer: client.clone(),
					_phantom: PhantomData,
				})
				.await
			},
		}
	}
}
