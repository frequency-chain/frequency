use crate::service::new_partial;
use cli_opt::SealingMode;
pub use futures::stream::StreamExt;
use sc_consensus::block_import::BlockImport;

use core::marker::PhantomData;
use futures::Stream;
use sc_client_api::backend::{Backend as ClientBackend, Finalizer};
use sc_consensus_manual_seal::{
	finalize_block, EngineCommand, FinalizeBlockParams, ManualSealParams, MANUAL_SEAL_ENGINE_ID,
};
use sc_service::{Configuration, TaskManager};
use sc_transaction_pool_api::TransactionPool;
use sp_api::{ProvideRuntimeApi, TransactionFor};
use sp_blockchain::HeaderBackend;
use sp_consensus::{Environment, Proposer, SelectChain};
use sp_inherents::CreateInherentDataProviders;
use sp_runtime::traits::Block as BlockT;
use std::{sync::Arc, task::Poll};

/// Function to start Frequency in dev mode without a relay chain
/// This function is called when --chain dev --sealing= is passed.
#[allow(clippy::expect_used)]
pub fn frequency_dev_sealing(
	config: Configuration,
	sealing_mode: SealingMode,
	sealing_interval: u16,
	sealing_create_empty_blocks: bool,
) -> Result<TaskManager, sc_service::error::Error> {
	let extra: String = if sealing_mode == SealingMode::Interval {
		format!(" ({}s interval)", sealing_interval)
	} else {
		String::from("")
	};
	log::info!("📎 Development mode (no relay chain) with {} sealing{}", sealing_mode, extra);

	let sc_service::PartialComponents {
		client,
		backend,
		mut task_manager,
		import_queue,
		keystore_container,
		select_chain: maybe_select_chain,
		transaction_pool,
		other: (_block_import, mut telemetry, _),
	} = new_partial(&config, true)?;

	// Build the network components required for the blockchain.
	let (network, system_rpc_tx, tx_handler_controller, network_starter, sync_service) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			block_announce_validator_builder: None,
			warp_sync_params: None,
		})?;

	// Start off-chain workers if enabled
	if config.offchain_worker.enabled {
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
				config.role.is_authority(),
				client.clone(),
				offchain_workers,
				task_manager.spawn_handle(),
				network.clone(),
			),
		);
	}

	let prometheus_registry = config.prometheus_registry().cloned();

	let role = config.role.clone();

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
					EngineCommand::SealNewBlock {
						create_empty: true,
						finalize: true,
						parent_hash: None,
						sender: None,
					}
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

		// Prepare the future for manual sealing block authoring
		let authorship_future = run_seal_command(
			sealing_mode,
			sealing_create_empty_blocks,
			sc_consensus_manual_seal::ManualSealParams {
				block_import: client.clone(),
				env: proposer_factory,
				client: client.clone(),
				pool: transaction_pool.clone(),
				commands_stream: futures::stream_select!(commands_stream, import_stream),
				select_chain,
				consensus_data_provider: None,
				create_inherent_data_providers: |_, _| async {
					Ok((sp_timestamp::InherentDataProvider::from_system_time(),))
				},
			},
		);

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
		config,
		keystore: keystore_container.keystore(),
		backend,
		network: network.clone(),
		sync_service: sync_service.clone(),
		system_rpc_tx,
		tx_handler_controller,
		telemetry: telemetry.as_mut(),
	})?;

	network_starter.start_network();

	Ok(task_manager)
}

/// Override manual sealing to handle creating non-empty blocks for interval sealing mode without nuisance warning logs
pub async fn run_seal_command<B, BI, CB, E, C, TP, SC, CS, CIDP, P>(
	sealing_mode: SealingMode,
	sealing_create_empty_blocks: bool,
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
	BI: BlockImport<B, Error = sp_consensus::Error, Transaction = sp_api::TransactionFor<C, B>>
		+ Send
		+ Sync
		+ 'static,
	C: HeaderBackend<B> + Finalizer<B, CB> + ProvideRuntimeApi<B> + 'static,
	CB: ClientBackend<B> + 'static,
	E: Environment<B> + 'static,
	E::Proposer: Proposer<B, Proof = P, Transaction = TransactionFor<C, B>>,
	CS: Stream<Item = EngineCommand<<B as BlockT>::Hash>> + Unpin + 'static,
	SC: SelectChain<B> + 'static,
	TransactionFor<C, B>: 'static,
	TP: TransactionPool<Block = B>,
	CIDP: CreateInherentDataProviders<B, ()>,
	P: Send + Sync + 'static,
{
	while let Some(command) = commands_stream.next().await {
		match command {
			EngineCommand::SealNewBlock { create_empty, finalize, parent_hash, sender } =>
				if sealing_mode != SealingMode::Interval ||
					sealing_create_empty_blocks ||
					pool.ready().count() > 0
				{
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
