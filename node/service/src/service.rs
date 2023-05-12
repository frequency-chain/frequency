#![allow(unused_imports)]
//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

// std
use std::{sync::Arc, time::Duration};

use cli_opt::SealingMode;

use cumulus_client_cli::CollatorOptions;
use frequency_runtime::RuntimeApi;

pub use futures::stream::StreamExt;

// RPC
use common_primitives::node::{AccountId, Balance, Block, Hash, Index as Nonce};
use jsonrpsee::RpcModule;

// Cumulus Imports
use cumulus_client_consensus_aura::{AuraConsensus, BuildAuraConsensusParams, SlotProportion};
use cumulus_client_consensus_common::{
	ParachainBlockImport as TParachainBlockImport, ParachainConsensus,
};
use cumulus_client_network::BlockAnnounceValidator;
use cumulus_client_service::{
	build_relay_chain_interface, prepare_node_config, start_collator, start_full_node,
	StartCollatorParams, StartFullNodeParams,
};
use cumulus_primitives_core::ParaId;
use cumulus_primitives_parachain_inherent::MockValidationDataInherentDataProvider;
use cumulus_relay_chain_interface::{RelayChainError, RelayChainInterface};

// Substrate Imports
use sc_consensus::{ImportQueue, LongestChain};
use sc_executor::NativeElseWasmExecutor;
use sc_network::NetworkService;
use sc_network_common::service::NetworkBlock;
use sc_service::{Configuration, PartialComponents, TFullBackend, TFullClient, TaskManager};
use sc_telemetry::{Telemetry, TelemetryHandle, TelemetryWorker, TelemetryWorkerHandle};
use sp_blockchain::HeaderBackend;
use sp_keystore::SyncCryptoStorePtr;
use substrate_prometheus_endpoint::Registry;

type FullBackend = TFullBackend<Block>;

type MaybeFullSelectChain = Option<LongestChain<FullBackend, Block>>;

/// Native executor instance for frequency.
pub mod frequency_executor {
	pub use frequency_runtime;
	use log::debug;
	use sc_executor::{
		sp_wasm_interface,
		sp_wasm_interface::{Function, HostFunctionRegistry, HostFunctions},
	};

	/// Native executor instance for frequency mainnet.
	pub struct FrequencyExecutorDispatch;

	impl sc_executor::NativeExecutionDispatch for FrequencyExecutorDispatch {
		#[cfg(feature = "runtime-benchmarks")]
		type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;

		#[cfg(not(feature = "runtime-benchmarks"))]
		type ExtendHostFunctions = ();

		fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
			frequency_runtime::api::dispatch(method, data)
		}

		fn native_version() -> sc_executor::NativeVersion {
			frequency_runtime::native_version()
		}
	}

	#[cfg(feature = "try-runtime")]
	impl HostFunctions for FrequencyExecutorDispatch {
		fn host_functions() -> Vec<&'static dyn Function> {
			Vec::new()
		}

		fn register_static<T: HostFunctionRegistry>(_registry: &mut T) -> Result<(), T::Error> {
			Ok(())
		}
	}
}

pub use frequency_executor::*;

type ParachainExecutor = NativeElseWasmExecutor<FrequencyExecutorDispatch>;

/// Frequency parachain
pub type ParachainClient = TFullClient<Block, RuntimeApi, ParachainExecutor>;

type ParachainBackend = TFullBackend<Block>;

type ParachainBlockImport = TParachainBlockImport<Block, Arc<ParachainClient>, ParachainBackend>;

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
pub fn new_partial(
	config: &Configuration,
	instant_sealing: bool,
) -> Result<
	PartialComponents<
		ParachainClient,
		ParachainBackend,
		MaybeFullSelectChain,
		sc_consensus::DefaultImportQueue<Block, ParachainClient>,
		sc_transaction_pool::FullPool<Block, ParachainClient>,
		(ParachainBlockImport, Option<Telemetry>, Option<TelemetryWorkerHandle>),
	>,
	sc_service::Error,
> {
	let telemetry = config
		.telemetry_endpoints
		.clone()
		.filter(|x| !x.is_empty())
		.map(|endpoints| -> Result<_, sc_telemetry::Error> {
			let worker = TelemetryWorker::new(16)?;
			let telemetry = worker.handle().new_telemetry(endpoints);
			Ok((worker, telemetry))
		})
		.transpose()?;

	let executor = ParachainExecutor::new(
		config.wasm_method,
		config.default_heap_pages,
		config.max_runtime_instances,
		config.runtime_cache_size,
	);

	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, _>(
			config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
			executor,
		)?;
	let client = Arc::new(client);

	let telemetry_worker_handle = telemetry.as_ref().map(|(worker, _)| worker.handle());

	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager.spawn_handle().spawn("telemetry", None, worker.run());
		telemetry
	});

	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_essential_handle(),
		client.clone(),
	);

	let block_import = ParachainBlockImport::new(client.clone(), backend.clone());

	let import_queue = build_import_queue(
		client.clone(),
		block_import.clone(),
		config,
		telemetry.as_ref().map(|telemetry| telemetry.handle()),
		&task_manager,
	)?;

	let select_chain =
		if instant_sealing { Some(LongestChain::new(backend.clone())) } else { None };

	let params = PartialComponents {
		backend,
		client,
		import_queue,
		keystore_container,
		task_manager,
		transaction_pool,
		select_chain,
		other: (block_import, telemetry, telemetry_worker_handle),
	};

	Ok(params)
}

/// Start a node with the given parachain `Configuration` and relay chain `Configuration`.
///
/// This is the actual implementation that is abstract over the executor and the runtime api.
#[allow(clippy::expect_used)]
#[sc_tracing::logging::prefix_logs_with("Parachain")]
#[cfg(any(not(feature = "frequency-no-relay"), feature = "frequency-lint-check"))]
async fn start_node_impl(
	parachain_config: Configuration,
	polkadot_config: Configuration,
	collator_options: CollatorOptions,
	id: ParaId,
	hwbench: Option<sc_sysinfo::HwBench>,
) -> sc_service::error::Result<(TaskManager, Arc<ParachainClient>)> {
	let parachain_config = prepare_node_config(parachain_config);

	let params = new_partial(&parachain_config, false)?;
	let (block_import, mut telemetry, telemetry_worker_handle) = params.other;

	let client = params.client.clone();
	let backend = params.backend.clone();
	let mut task_manager = params.task_manager;

	let (relay_chain_interface, collator_key) = build_relay_chain_interface(
		polkadot_config,
		&parachain_config,
		telemetry_worker_handle,
		&mut task_manager,
		collator_options.clone(),
		hwbench.clone(),
	)
	.await
	.map_err(|e| match e {
		RelayChainError::ServiceError(polkadot_service::Error::Sub(x)) => x,
		s => s.to_string().into(),
	})?;

	let block_announce_validator = BlockAnnounceValidator::new(relay_chain_interface.clone(), id);

	let force_authoring = parachain_config.force_authoring;
	let validator = parachain_config.role.is_authority();
	let prometheus_registry = parachain_config.prometheus_registry().cloned();
	let transaction_pool = params.transaction_pool.clone();
	let import_queue_service = params.import_queue.service();
	let (network, system_rpc_tx, tx_handler_controller, start_network) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &parachain_config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue: params.import_queue,
			block_announce_validator_builder: Some(Box::new(|_| {
				Box::new(block_announce_validator)
			})),
			warp_sync: None,
		})?;

	let rpc_builder = {
		let client = client.clone();
		let transaction_pool = transaction_pool.clone();

		Box::new(move |deny_unsafe, _| {
			let deps = crate::rpc::FullDeps {
				client: client.clone(),
				pool: transaction_pool.clone(),
				deny_unsafe,
				command_sink: None,
			};

			crate::rpc::create_full(deps).map_err(Into::into)
		})
	};

	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		rpc_builder,
		client: client.clone(),
		transaction_pool: transaction_pool.clone(),
		task_manager: &mut task_manager,
		config: parachain_config,
		keystore: params.keystore_container.sync_keystore(),
		backend: backend.clone(),
		network: network.clone(),
		system_rpc_tx,
		tx_handler_controller,
		telemetry: telemetry.as_mut(),
	})?;

	if let Some(hwbench) = hwbench {
		sc_sysinfo::print_hwbench(&hwbench);

		if let Some(ref mut telemetry) = telemetry {
			let telemetry_handle = telemetry.handle();
			task_manager.spawn_handle().spawn(
				"telemetry_hwbench",
				None,
				sc_sysinfo::initialize_hwbench_telemetry(telemetry_handle, hwbench),
			);
		}
	}

	let announce_block = {
		let network = network.clone();
		Arc::new(move |hash, data| network.announce_block(hash, data))
	};

	let relay_chain_slot_duration = Duration::from_secs(6);

	if validator {
		let parachain_consensus = build_consensus(
			client.clone(),
			block_import,
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|t| t.handle()),
			&task_manager,
			relay_chain_interface.clone(),
			transaction_pool,
			network,
			params.keystore_container.sync_keystore(),
			force_authoring,
			id,
		)?;

		let spawner = task_manager.spawn_handle();

		let params = StartCollatorParams {
			para_id: id,
			block_status: client.clone(),
			announce_block,
			client: client.clone(),
			task_manager: &mut task_manager,
			relay_chain_interface,
			spawner,
			parachain_consensus,
			import_queue: import_queue_service,
			collator_key: collator_key.expect("Command line arguments do not allow this. qed"),
			relay_chain_slot_duration,
		};

		start_collator(params).await?;
	} else {
		let params = StartFullNodeParams {
			client: client.clone(),
			announce_block,
			task_manager: &mut task_manager,
			para_id: id,
			relay_chain_interface,
			relay_chain_slot_duration,
			import_queue: import_queue_service,
		};

		start_full_node(params)?;
	}

	start_network.start_network();

	Ok((task_manager, client))
}

/// Build the import queue for the parachain runtime.
fn build_import_queue(
	client: Arc<ParachainClient>,
	block_import: ParachainBlockImport,
	config: &Configuration,
	telemetry: Option<TelemetryHandle>,
	task_manager: &TaskManager,
) -> Result<sc_consensus::DefaultImportQueue<Block, ParachainClient>, sc_service::Error> {
	let slot_duration = cumulus_client_consensus_aura::slot_duration(&*client)?;

	cumulus_client_consensus_aura::import_queue::<
		sp_consensus_aura::sr25519::AuthorityPair,
		_,
		_,
		_,
		_,
		_,
	>(cumulus_client_consensus_aura::ImportQueueParams {
		block_import,
		client: client.clone(),
		create_inherent_data_providers: move |_, _| async move {
			let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

			let slot =
				sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
					*timestamp,
					slot_duration,
				);

			Ok((slot, timestamp))
		},
		registry: config.prometheus_registry(),
		spawner: &task_manager.spawn_essential_handle(),
		telemetry,
	})
	.map_err(Into::into)
}

#[cfg(any(not(feature = "frequency-no-relay"), feature = "frequency-lint-check"))]
fn build_consensus(
	client: Arc<ParachainClient>,
	block_import: ParachainBlockImport,
	prometheus_registry: Option<&Registry>,
	telemetry: Option<TelemetryHandle>,
	task_manager: &TaskManager,
	relay_chain_interface: Arc<dyn RelayChainInterface>,
	transaction_pool: Arc<sc_transaction_pool::FullPool<Block, ParachainClient>>,
	sync_oracle: Arc<NetworkService<Block, Hash>>,
	keystore: SyncCryptoStorePtr,
	force_authoring: bool,
	id: ParaId,
) -> Result<Box<dyn ParachainConsensus<Block>>, sc_service::Error> {
	let slot_duration = cumulus_client_consensus_aura::slot_duration(&*client)?;

	let proposer_factory = sc_basic_authorship::ProposerFactory::with_proof_recording(
		task_manager.spawn_handle(),
		client.clone(),
		transaction_pool,
		prometheus_registry,
		telemetry.clone(),
	);

	let params = BuildAuraConsensusParams {
		proposer_factory,
		create_inherent_data_providers: move |_, (relay_parent, validation_data)| {
			let relay_chain_interface = relay_chain_interface.clone();
			async move {
				let parachain_inherent =
					cumulus_primitives_parachain_inherent::ParachainInherentData::create_at(
						relay_parent,
						&relay_chain_interface,
						&validation_data,
						id,
					)
					.await;
				let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

				let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
							*timestamp,
							slot_duration,
						);

				let parachain_inherent = parachain_inherent.ok_or_else(|| {
					Box::<dyn std::error::Error + Send + Sync>::from(
						"Failed to create parachain inherent",
					)
				})?;
				Ok((slot, timestamp, parachain_inherent))
			}
		},
		block_import,
		para_client: client,
		backoff_authoring_blocks: Option::<()>::None,
		sync_oracle,
		keystore,
		force_authoring,
		slot_duration,
		// We got around 500ms for proposing
		block_proposal_slot_portion: SlotProportion::new(1f32 / 24f32),
		// And a maximum of 750ms if slots are skipped
		max_block_proposal_slot_portion: Some(SlotProportion::new(1f32 / 16f32)),
		telemetry,
	};

	Ok(AuraConsensus::build::<sp_consensus_aura::sr25519::AuthorityPair, _, _, _, _, _, _>(params))
}

/// Start a parachain node.
#[cfg(any(not(feature = "frequency-no-relay"), feature = "frequency-lint-check"))]
pub async fn start_parachain_node(
	parachain_config: Configuration,
	polkadot_config: Configuration,
	collator_options: CollatorOptions,
	id: ParaId,
	hwbench: Option<sc_sysinfo::HwBench>,
) -> sc_service::error::Result<(TaskManager, Arc<ParachainClient>)> {
	start_node_impl(parachain_config, polkadot_config, collator_options, id, hwbench).await
}

/// Function to start frequency parachain with instant sealing in dev mode.
/// This function is called when --chain dev --sealing=instant is passed.
#[allow(clippy::expect_used)]
#[cfg(feature = "frequency-no-relay")]
pub fn frequency_dev_sealing(
	config: Configuration,
	sealing_mode: SealingMode,
) -> Result<TaskManager, sc_service::error::Error> {
	log::info!("📎 Development mode (no relay chain) with {} sealing.", sealing_mode);

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

	if parachain_config.offchain_worker.enabled {
		let offchain_workers = Arc::new(sc_offchain::OffchainWorkers::new_with_options(
			client.clone(),
			sc_offchain::OffchainWorkerOptions { enable_http_requests: false },
		));

		// Start the offchain workers to have
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
		let import_stream = pool
			.validated_pool()
			.import_notification_stream()
			.filter(move |_| futures::future::ready(sealing_mode == SealingMode::Instant))
			.map(|_| sc_consensus_manual_seal::rpc::EngineCommand::SealNewBlock {
				create_empty: true,
				finalize: true,
				parent_hash: None,
				sender: None,
			});

		let client_for_cidp = client.clone();
		let block_import = ParachainBlockImport::new(client.clone(), backend.clone());

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
		// we spawn the future on a background thread managed by service.
		task_manager.spawn_essential_handle().spawn_blocking(
			if sealing_mode == SealingMode::Instant { "instant-seal" } else { "manual-seal" },
			Some("block-authoring"),
			authorship_future,
		);
		Some(command_sink)
	} else {
		None
	};

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
