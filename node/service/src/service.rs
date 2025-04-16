#![allow(unused_imports)]
//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

// Originally from https://github.com/paritytech/cumulus/blob/master/parachain-template/node/src/service.rs

// std
use cumulus_client_cli::CollatorOptions;
use frequency_runtime::RuntimeApi;
use sc_client_api::Backend;
use std::{ptr::addr_eq, sync::Arc, time::Duration};

// RPC
use common_primitives::node::{AccountId, Balance, Block, Hash, Index as Nonce};
use jsonrpsee::RpcModule;
use substrate_prometheus_endpoint::Registry;

// Cumulus Imports
use cumulus_client_collator::service::CollatorService;
use cumulus_client_consensus_aura::{AuraConsensus, SlotProportion};
use cumulus_client_consensus_common::{
	ParachainBlockImport as TParachainBlockImport, ParachainConsensus,
};
use cumulus_client_consensus_proposer::Proposer;
use cumulus_client_network::RequireSecondedInBlockAnnounce;
use cumulus_client_service::{
	build_network, build_relay_chain_interface, prepare_node_config, start_relay_chain_tasks,
	BuildNetworkParams, CollatorSybilResistance, DARecoveryProfile, StartCollatorParams,
	StartFullNodeParams, StartRelayChainTasksParams,
};
use cumulus_primitives_core::{
	relay_chain::{CollatorPair, ValidationCode},
	ParaId,
};
use cumulus_relay_chain_interface::{OverseerHandle, RelayChainError, RelayChainInterface};

// Substrate Imports
use frame_benchmarking_cli::SUBSTRATE_REFERENCE_HARDWARE;
use futures::FutureExt;
use sc_consensus::{ImportQueue, LongestChain};
use sc_executor::{HeapAllocStrategy, WasmExecutor, DEFAULT_HEAP_ALLOC_STRATEGY};

use sc_network::{config::FullNetworkConfiguration, NetworkBackend, NetworkBlock, NetworkService};
use sc_network_sync::SyncingService;
use sc_service::{Configuration, PartialComponents, TFullBackend, TFullClient, TaskManager};
use sc_telemetry::{Telemetry, TelemetryHandle, TelemetryWorker, TelemetryWorkerHandle};
use sc_transaction_pool_api::OffchainTransactionPoolFactory;
use sp_blockchain::HeaderBackend;
use sp_keystore::KeystorePtr;

use common_runtime::prod_or_testnet_or_local;

type FullBackend = TFullBackend<Block>;

type MaybeFullSelectChain = Option<LongestChain<FullBackend, Block>>;

#[cfg(not(feature = "runtime-benchmarks"))]
type HostFunctions = (
	cumulus_client_service::ParachainHostFunctions,
	common_primitives::offchain::custom::HostFunctions,
);

#[cfg(feature = "runtime-benchmarks")]
type HostFunctions = (
	cumulus_client_service::ParachainHostFunctions,
	frame_benchmarking::benchmarking::HostFunctions,
	common_primitives::offchain::custom::HostFunctions,
);

use crate::common::start_offchain_workers;
pub use frequency_runtime;

type ParachainExecutor = WasmExecutor<HostFunctions>;

/// Frequency parachain
pub type ParachainClient = TFullClient<Block, RuntimeApi, ParachainExecutor>;

type ParachainBackend = TFullBackend<Block>;

type ParachainBlockImport = TParachainBlockImport<Block, Arc<ParachainClient>, ParachainBackend>;

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
#[allow(deprecated)]
pub fn new_partial(
	config: &Configuration,
	instant_sealing: bool,
) -> Result<
	PartialComponents<
		ParachainClient,
		ParachainBackend,
		MaybeFullSelectChain,
		sc_consensus::DefaultImportQueue<Block>,
		sc_transaction_pool::TransactionPoolHandle<Block, ParachainClient>,
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

	let heap_pages = config
		.executor
		.default_heap_pages
		.map_or(DEFAULT_HEAP_ALLOC_STRATEGY, |h| HeapAllocStrategy::Static { extra_pages: h as _ });

	let executor = ParachainExecutor::builder()
		.with_execution_method(config.executor.wasm_method)
		.with_onchain_heap_alloc_strategy(heap_pages)
		.with_offchain_heap_alloc_strategy(heap_pages)
		.with_max_runtime_instances(config.executor.max_runtime_instances)
		.with_runtime_cache_size(config.executor.runtime_cache_size)
		.build();

	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts_record_import::<Block, RuntimeApi, _>(
			config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
			executor,
			true,
		)?;
	let client = Arc::new(client);

	let telemetry_worker_handle = telemetry.as_ref().map(|(worker, _)| worker.handle());

	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager.spawn_handle().spawn("telemetry", None, worker.run());
		telemetry
	});

	// See https://github.com/paritytech/polkadot-sdk/pull/4639 for how to enable the fork-aware pool.
	let transaction_pool = Arc::from(
		sc_transaction_pool::Builder::new(
			task_manager.spawn_essential_handle(),
			client.clone(),
			config.role.is_authority().into(),
		)
		.with_options(config.transaction_pool.clone())
		.with_prometheus(config.prometheus_registry())
		.build(),
	);

	let block_import = ParachainBlockImport::new(client.clone(), backend.clone());

	#[cfg(feature = "frequency-no-relay")]
	let import_queue = sc_consensus_manual_seal::import_queue(
		Box::new(client.clone()),
		&task_manager.spawn_essential_handle(),
		config.prometheus_registry(),
	);

	#[cfg(not(feature = "frequency-no-relay"))]
	let import_queue = build_import_queue(
		client.clone(),
		block_import.clone(),
		config,
		telemetry.as_ref().map(|telemetry| telemetry.handle()),
		&task_manager,
	);

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
pub async fn start_parachain_node(
	parachain_config: Configuration,
	polkadot_config: Configuration,
	collator_options: CollatorOptions,
	para_id: ParaId,
	hwbench: Option<sc_sysinfo::HwBench>,
) -> sc_service::error::Result<(TaskManager, Arc<ParachainClient>)> {
	use crate::common::listen_addrs_to_normalized_strings;
	use common_primitives::offchain::OcwCustomExt;
	use sc_client_db::Backend;

	let parachain_config = prepare_node_config(parachain_config);

	let params = new_partial(&parachain_config, false)?;
	let (block_import, mut telemetry, telemetry_worker_handle) = params.other;

	let prometheus_registry = parachain_config.prometheus_registry().cloned();
	let net_config = FullNetworkConfiguration::<_, _, sc_network::NetworkWorker<Block, Hash>>::new(
		&parachain_config.network,
		prometheus_registry,
	);

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
	.map_err(|e| sc_service::Error::Application(Box::new(e) as Box<_>))?;

	let validator = parachain_config.role.is_authority();
	let prometheus_registry = parachain_config.prometheus_registry().cloned();
	let transaction_pool = params.transaction_pool.clone();
	let import_queue_service = params.import_queue.service();

	// NOTE: because we use Aura here explicitly, we can use `CollatorSybilResistance::Resistant`
	// when starting the network.
	let (network, system_rpc_tx, tx_handler_controller, sync_service) =
		build_network(BuildNetworkParams {
			parachain_config: &parachain_config,
			net_config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			para_id,
			spawn_handle: task_manager.spawn_handle(),
			relay_chain_interface: relay_chain_interface.clone(),
			import_queue: params.import_queue,
			sybil_resistance_level: CollatorSybilResistance::Resistant,
		})
		.await?;

	// Start off-chain workers if enabled
	if parachain_config.offchain_worker.enabled {
		log::info!("OFFCHAIN WORKER is Enabled!");
		start_offchain_workers(
			&client,
			&parachain_config,
			Some(params.keystore_container.keystore()),
			&backend,
			Some(OffchainTransactionPoolFactory::new(transaction_pool.clone())),
			Arc::new(network.clone()),
			&task_manager,
		);
	}

	let rpc_builder = {
		let client = client.clone();
		let transaction_pool = transaction_pool.clone();

		let backend = match parachain_config.offchain_worker.enabled {
			true => backend.offchain_storage(),
			false => None,
		};

		Box::new(move |_| {
			let deps = crate::rpc::FullDeps {
				client: client.clone(),
				pool: transaction_pool.clone(),
				command_sink: None,
			};

			crate::rpc::create_full(deps, backend.clone()).map_err(Into::into)
		})
	};

	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		rpc_builder,
		client: client.clone(),
		transaction_pool: transaction_pool.clone(),
		task_manager: &mut task_manager,
		config: parachain_config,
		keystore: params.keystore_container.keystore(),
		backend: backend.clone(),
		network: network.clone(),
		sync_service: sync_service.clone(),
		system_rpc_tx,
		tx_handler_controller,
		telemetry: telemetry.as_mut(),
	})?;

	if let Some(hwbench) = hwbench {
		sc_sysinfo::print_hwbench(&hwbench);
		let is_rc_authority = false;
		// Here you can check whether the hardware meets your chains' requirements. Putting a link
		// in there and swapping out the requirements for your own are probably a good idea. The
		// requirements for a para-chain are dictated by its relay-chain.
		if validator &&
			SUBSTRATE_REFERENCE_HARDWARE.check_hardware(&hwbench, is_rc_authority).is_err()
		{
			log::warn!(
				"⚠️  The hardware does not meet the minimal requirements for role 'Authority'."
			);
		}

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
		let sync_service = sync_service.clone();
		Arc::new(move |hash, data| sync_service.announce_block(hash, data))
	};

	let relay_chain_slot_duration = Duration::from_secs(6);

	let overseer_handle = relay_chain_interface
		.overseer_handle()
		.map_err(|e| sc_service::Error::Application(Box::new(e)))?;

	start_relay_chain_tasks(StartRelayChainTasksParams {
		client: client.clone(),
		announce_block: announce_block.clone(),
		para_id,
		relay_chain_interface: relay_chain_interface.clone(),
		task_manager: &mut task_manager,
		da_recovery_profile: if validator {
			DARecoveryProfile::Collator
		} else {
			DARecoveryProfile::FullNode
		},
		import_queue: import_queue_service,
		relay_chain_slot_duration,
		recovery_handle: Box::new(overseer_handle.clone()),
		sync_service: sync_service.clone(),
	})?;

	if validator {
		start_consensus(
			client.clone(),
			backend.clone(),
			block_import,
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|t| t.handle()),
			&task_manager,
			relay_chain_interface.clone(),
			transaction_pool,
			sync_service.clone(),
			params.keystore_container.keystore(),
			relay_chain_slot_duration,
			para_id,
			collator_key.expect("Command line arguments do not allow this. qed"),
			overseer_handle,
			announce_block,
		)?;
	}

	Ok((task_manager, client))
}

/// Build the import queue for the parachain runtime.
#[cfg(not(feature = "frequency-no-relay"))]
/// Build the import queue for the parachain runtime.
fn build_import_queue(
	client: Arc<ParachainClient>,
	block_import: ParachainBlockImport,
	config: &Configuration,
	telemetry: Option<TelemetryHandle>,
	task_manager: &TaskManager,
) -> sc_consensus::DefaultImportQueue<Block> {
	cumulus_client_consensus_aura::equivocation_import_queue::fully_verifying_import_queue::<
		sp_consensus_aura::sr25519::AuthorityPair,
		_,
		_,
		_,
		_,
	>(
		client,
		block_import,
		move |_, _| async move {
			let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
			Ok(timestamp)
		},
		&task_manager.spawn_essential_handle(),
		config.prometheus_registry(),
		telemetry,
	)
}

#[cfg(any(not(feature = "frequency-no-relay"), feature = "frequency-lint-check"))]
fn start_consensus(
	client: Arc<ParachainClient>,
	backend: Arc<ParachainBackend>,
	block_import: ParachainBlockImport,
	prometheus_registry: Option<&Registry>,
	telemetry: Option<TelemetryHandle>,
	task_manager: &TaskManager,
	relay_chain_interface: Arc<dyn RelayChainInterface>,
	transaction_pool: Arc<sc_transaction_pool::TransactionPoolHandle<Block, ParachainClient>>,
	_sync_oracle: Arc<SyncingService<Block>>,
	keystore: KeystorePtr,
	relay_chain_slot_duration: Duration,
	para_id: ParaId,
	collator_key: CollatorPair,
	overseer_handle: OverseerHandle,
	announce_block: Arc<dyn Fn(Hash, Option<Vec<u8>>) + Send + Sync>,
) -> Result<(), sc_service::Error> {
	use cumulus_client_consensus_aura::collators::lookahead::{self as aura, Params as AuraParams};

	let proposer_factory = sc_basic_authorship::ProposerFactory::with_proof_recording(
		task_manager.spawn_handle(),
		client.clone(),
		transaction_pool,
		prometheus_registry,
		telemetry.clone(),
	);

	let proposer = Proposer::new(proposer_factory);

	let collator_service = CollatorService::new(
		client.clone(),
		Arc::new(task_manager.spawn_handle()),
		announce_block,
		client.clone(),
	);

	let params = AuraParams {
		create_inherent_data_providers: move |_, ()| async move { Ok(()) },
		block_import,
		para_client: client.clone(),
		para_backend: backend.clone(),
		relay_client: relay_chain_interface,
		code_hash_provider: move |block_hash| {
			client.code_at(block_hash).ok().map(|c| ValidationCode::from(c).hash())
		},
		keystore,
		collator_key,
		para_id,
		overseer_handle,
		relay_chain_slot_duration,
		proposer,
		collator_service,
		authoring_duration: Duration::from_millis(prod_or_testnet_or_local!(500, 2000, 2000)),
		reinitialize: false,
		max_pov_percentage: None, // default 50%
	};

	let fut = aura::run::<Block, sp_consensus_aura::sr25519::AuthorityPair, _, _, _, _, _, _, _, _>(
		params,
	);

	task_manager.spawn_essential_handle().spawn("aura", None, fut);

	Ok(())
}
