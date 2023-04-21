use crate::{
	benchmarking::{inherent_benchmark_data, RemarkBuilder},
	cli::{Cli, RelayChainCli, Subcommand},
};
use codec::Encode;
use common_primitives::node::Block;
use cumulus_client_cli::generate_genesis_block;
use cumulus_primitives_core::ParaId;
use frame_benchmarking_cli::BenchmarkCmd;
use frequency_service::{
	chain_spec,
	service::{
		frequency_runtime::VERSION, new_partial, FrequencyExecutorDispatch as ExecutorDispatch,
	},
};
use log::info;
use sc_cli::{
	ChainSpec, CliConfiguration, DefaultConfigurationValues, ImportParams, KeystoreParams,
	NetworkParams, Result, RuntimeVersion, SharedParams, SubstrateCli,
};
use sc_service::config::{BasePath, PrometheusConfig};
use sp_core::hexdisplay::HexDisplay;
use sp_runtime::traits::{AccountIdConversion, Block as BlockT};
use std::net::SocketAddr;

enum ChainIdentity {
	Frequency,
	FrequencyRococo,
	FrequencyLocal,
}

trait IdentifyChain {
	fn identify(&self) -> ChainIdentity;
}

impl IdentifyChain for dyn sc_service::ChainSpec {
	fn identify(&self) -> ChainIdentity {
		if self.id() == "frequency" {
			ChainIdentity::Frequency
		} else if self.id() == "frequency-rococo" {
			ChainIdentity::FrequencyRococo
		} else if self.id() == "frequency-local" {
			ChainIdentity::FrequencyLocal
		} else {
			panic!("Unknown chain identity")
		}
	}
}

impl PartialEq for ChainIdentity {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(ChainIdentity::Frequency, ChainIdentity::Frequency) => true,
			(ChainIdentity::FrequencyRococo, ChainIdentity::FrequencyRococo) => true,
			(ChainIdentity::FrequencyLocal, ChainIdentity::FrequencyLocal) => true,
			_ => false,
		}
	}
}

impl<T: sc_service::ChainSpec + 'static> IdentifyChain for T {
	fn identify(&self) -> ChainIdentity {
		<dyn sc_service::ChainSpec>::identify(self)
	}
}

fn load_spec(id: &str) -> std::result::Result<Box<dyn ChainSpec>, String> {
	match id {
		#[cfg(feature = "frequency")]
		"frequency-bench" => return Ok(Box::new(chain_spec::frequency::benchmark_mainnet_config())),
		#[cfg(feature = "frequency")]
		"frequency" => return Ok(Box::new(chain_spec::frequency::load_frequency_spec())),
		#[cfg(feature = "frequency-rococo-local")]
		"frequency-local" | "dev" =>
			return Ok(Box::new(chain_spec::frequency_rococo::local_testnet_config())),
		#[cfg(feature = "frequency-rococo-testnet")]
		"frequency-rococo" | "rococo" | "testnet" =>
			return Ok(Box::new(chain_spec::frequency_rococo::load_frequency_rococo_spec())),
		path => {
			if path.is_empty() {
				if cfg!(feature = "frequency") {
					#[cfg(feature = "frequency")]
					{
						return Ok(Box::new(chain_spec::frequency::load_frequency_spec()))
					}
					#[cfg(not(feature = "frequency"))]
					return Err("Frequency runtime is not available.".into())
				} else if cfg!(feature = "frequency-rococo-local") {
					#[cfg(feature = "frequency-rococo-local")]
					{
						return Ok(Box::new(chain_spec::frequency_rococo::local_testnet_config()))
					}
					#[cfg(not(feature = "frequency-rococo-local"))]
					return Err("Frequency Local runtime is not available.".into())
				} else if cfg!(feature = "frequency-rococo-testnet") {
					#[cfg(feature = "frequency-rococo-testnet")]
					{
						return Ok(Box::new(
							chain_spec::frequency_rococo::load_frequency_rococo_spec(),
						))
					}
					#[cfg(not(feature = "frequency-rococo-testnet"))]
					return Err("Frequency Rococo runtime is not available.".into())
				} else {
					return Err("No chain spec is available.".into())
				}
			}
			let path_buf = std::path::PathBuf::from(path);
			let spec = Box::new(chain_spec::DummyChainSpec::from_json_file(path_buf.clone())?)
				as Box<dyn ChainSpec>;
			if ChainIdentity::Frequency == spec.identify() {
				#[cfg(feature = "frequency")]
				{
					return Ok(Box::new(chain_spec::frequency::ChainSpec::from_json_file(path_buf)?))
				}
				#[cfg(not(feature = "frequency"))]
				return Err("Frequency runtime is not available.".into())
			} else if ChainIdentity::FrequencyRococo == spec.identify() {
				#[cfg(feature = "frequency-rococo-testnet")]
				{
					return Ok(Box::new(chain_spec::frequency_rococo::ChainSpec::from_json_file(
						path_buf,
					)?))
				}
				#[cfg(not(feature = "frequency-rococo-testnet"))]
				return Err("Frequency Rococo runtime is not available.".into())
			} else if ChainIdentity::FrequencyLocal == spec.identify() {
				#[cfg(feature = "frequency-rococo-local")]
				{
					return Ok(Box::new(chain_spec::frequency_rococo::ChainSpec::from_json_file(
						path_buf,
					)?))
				}
				#[cfg(not(feature = "frequency-rococo-local"))]
				return Err("Frequency Local runtime is not available.".into())
			} else {
				return Err("Unknown chain spec.".into())
			}
		},
	}
}

fn chain_name() -> String {
	"Frequency".into()
}

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		format!("{} Node", chain_name())
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		"Frequency\n\nThe command-line arguments provided first will be \
		passed to the parachain node, while the arguments provided after -- will be passed \
		to the relay chain node.\n\n\
		frequency <parachain-args> -- <relay-chain-args>"
			.into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/libertydsnp/frequency/issues/new".into()
	}

	fn copyright_start_year() -> i32 {
		2020
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		load_spec(id)
	}

	fn native_runtime_version(_spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		&VERSION
	}
}

impl SubstrateCli for RelayChainCli {
	fn impl_name() -> String {
		"Frequency".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		"Frequency\n\nThe command-line arguments provided first will be \
		passed to the parachain node, while the arguments provided after -- will be passed \
		to the relay chain node.\n\n\
		frequency <parachain-args> -- <relay-chain-args>"
			.into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/paritytech/cumulus/issues/new".into()
	}

	fn copyright_start_year() -> i32 {
		2020
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		polkadot_cli::Cli::from_iter([RelayChainCli::executable_name()].iter()).load_spec(id)
	}

	fn native_runtime_version(chain_spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		polkadot_cli::Cli::native_runtime_version(chain_spec)
	}
}

macro_rules! construct_async_run {
	(|$components:ident, $cli:ident, $cmd:ident, $config:ident| $( $code:tt )* ) => {{
		let runner = $cli.create_runner($cmd)?;
		runner.async_run(|$config| {
				let $components = new_partial(&$config, false)?;
				let task_manager = $components.task_manager;
				{ $( $code )* }.map(|v| (v, task_manager))
			})
	}}
}

/// Parse command line arguments into service configuration.
pub fn run() -> Result<()> {
	let cli = Cli::from_args();

	match &cli.subcommand {
		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		},
		Some(Subcommand::CheckBlock(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, components.import_queue))
			})
		},
		Some(Subcommand::ExportBlocks(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, config.database))
			})
		},
		Some(Subcommand::ExportState(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, config.chain_spec))
			})
		},
		Some(Subcommand::ImportBlocks(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, components.import_queue))
			})
		},
		Some(Subcommand::ExportMetadata(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| Ok(cmd.run(components.client)))
		},
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			runner.sync_run(|config| {
				let polkadot_cli = RelayChainCli::new(
					&config,
					[RelayChainCli::executable_name()].iter().chain(cli.relay_chain_args.iter()),
				);

				let polkadot_config = SubstrateCli::create_configuration(
					&polkadot_cli,
					&polkadot_cli,
					config.tokio_handle.clone(),
				)
				.map_err(|err| format!("Relay chain argument error: {}", err))?;

				cmd.run(config, polkadot_config)
			})
		},
		Some(Subcommand::Revert(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, components.backend, None))
			})
		},
		Some(Subcommand::ExportGenesisState(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|_config| {
				let spec = cli.load_spec(&cmd.shared_params.chain.clone().unwrap_or_default())?;
				let state_version = Cli::native_runtime_version(&spec).state_version();
				cmd.run::<Block>(&*spec, state_version)
			})
		},
		Some(Subcommand::ExportGenesisWasm(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|_config| {
				let spec = cli.load_spec(&cmd.shared_params.chain.clone().unwrap_or_default())?;
				cmd.run(&*spec)
			})
		},
		Some(Subcommand::Benchmark(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			match cmd {
				BenchmarkCmd::Pallet(cmd) =>
					if cfg!(feature = "runtime-benchmarks") {
						runner.sync_run(|config| cmd.run::<Block, ExecutorDispatch>(config))
					} else {
						return Err("Benchmarking wasn't enabled when building the node. \
									You can enable it with `--features runtime-benchmarks`."
							.into())
					},
				BenchmarkCmd::Block(cmd) => runner.sync_run(|config| {
					let partials = new_partial(&config, false)?;
					cmd.run(partials.client)
				}),
				#[cfg(not(feature = "runtime-benchmarks"))]
				BenchmarkCmd::Storage(_) =>
					return Err(sc_cli::Error::Input(
						"Compile with --features=runtime-benchmarks \
						to enable storage benchmarks."
							.into(),
					)
					.into()),
				#[cfg(feature = "runtime-benchmarks")]
				BenchmarkCmd::Storage(cmd) => runner.sync_run(|config| {
					let partials = new_partial(&config, false)?;
					let db = partials.backend.expose_db();
					let storage = partials.backend.expose_storage();

					cmd.run(config, partials.client.clone(), db, storage)
				}),
				BenchmarkCmd::Overhead(cmd) => runner.sync_run(|config| {
					let partials = new_partial(&config, false)?;
					let ext_builder = RemarkBuilder::new(partials.client.clone());

					cmd.run(
						config,
						partials.client,
						inherent_benchmark_data()?,
						Vec::new(),
						&ext_builder,
					)
				}),
				BenchmarkCmd::Machine(cmd) => runner.sync_run(|config| {
					cmd.run(&config, frame_benchmarking_cli::SUBSTRATE_REFERENCE_HARDWARE.clone())
				}),
				BenchmarkCmd::Extrinsic(_cmd) =>
					Err("Benchmarking command not implemented.".into()),
			}
		},

		#[cfg(feature = "try-runtime")]
		Some(Subcommand::TryRuntime(cmd)) => {
			use sc_executor::{sp_wasm_interface::ExtendedHostFunctions, NativeExecutionDispatch};
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				// we don't need any of the components of new_partial, just a runtime, or a task
				// manager to do `async_run`.
				let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
				let task_manager =
					sc_service::TaskManager::new(config.tokio_handle.clone(), registry)
						.map_err(|e| sc_cli::Error::Service(sc_service::Error::Prometheus(e)))?;
				Ok((
					cmd.run::<Block, ExtendedHostFunctions<
						sp_io::SubstrateHostFunctions,
						<ExecutorDispatch as NativeExecutionDispatch>::ExtendHostFunctions,
					>>(),
					task_manager,
				))
			})
		},
		Some(Subcommand::ExportRuntimeVersion(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			runner.async_run(|config| {
				let version = Cli::native_runtime_version(&config.chain_spec);
				// grab the task manager.
				let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
				let task_manager =
					sc_service::TaskManager::new(config.tokio_handle.clone(), registry)
						.map_err(|e| format!("Error: {:?}", e))?;
				Ok((cmd.run(version), task_manager))
			})
		},
		None => {
			#[cfg(feature = "frequency-rococo-local")]
			run_local(cli);
			#[cfg(not(feature = "frequency-rococo-local"))]
			run_parachain(cli);
			Ok(())
		},
	}
}

#[cfg(feature = "frequency-rococo-local")]
fn run_local(cli: Cli) -> Result<()> {
	let runner = cli.create_runner(&cli.run.normalize())?;
	runner.run_node_until_exit(|config| async move {
		if cli.instant_sealing {
			return frequency_service::service::frequency_dev_instant_sealing(config, true)
				.map_err(Into::into)
		} else if cli.manual_sealing {
			return frequency_service::service::frequency_dev_instant_sealing(config, false)
				.map_err(Into::into)
		}
	});
}

#[cfg(not(feature = "frequency-rococo-local"))]
fn run_parachain(cli: Cli) -> Result<()> {
	let runner = cli.create_runner(&cli.run.normalize())?;
	runner.run_node_until_exit(|config| async move {
		let para_id = chain_spec::Extensions::try_get(&*config.chain_spec)
			.map(|e| e.para_id)
			.ok_or("Could not find parachain ID in chain-spec.")?;
		let id = ParaId::from(para_id);

		let parachain_account =
			AccountIdConversion::<polkadot_primitives::v2::AccountId>::into_account_truncating(&id);

		let state_version = Cli::native_runtime_version(&config.chain_spec).state_version();
		let block: Block = generate_genesis_block(&*config.chain_spec, state_version)
			.map_err(|e| format!("{:?}", e))?;
		let genesis_state = format!("0x{:?}", HexDisplay::from(&block.header().encode()));

		info!("Parachain id: {:?}", id);
		info!("Parachain Account: {}", parachain_account);
		info!("Parachain genesis state: {}", genesis_state);
		info!("Is collating: {}", if config.role.is_authority() { "yes" } else { "no" });

		let tokio_handle = config.tokio_handle.clone();
		let polkadot_cli = RelayChainCli::new(
			&config,
			[RelayChainCli::executable_name()].iter().chain(cli.relay_chain_args.iter()),
		);
		let polkadot_config =
			SubstrateCli::create_configuration(&polkadot_cli, &polkadot_cli, tokio_handle)
				.map_err(|err| format!("Relay chain argument error: {}", err))?;

		let collator_options = cli.run.collator_options();
		let hwbench = if !cli.no_hardware_benchmarks {
			config.database.path().map(|database_path| {
				let _ = std::fs::create_dir_all(&database_path);
				sc_sysinfo::gather_hwbench(Some(database_path))
			})
		} else {
			None
		};
		return frequency_service::service::start_parachain_node(
			config,
			polkadot_config,
			collator_options,
			id,
			hwbench,
		)
		.await
		.map(|r| r.0)
		.map_err(Into::into)
	});
}

impl DefaultConfigurationValues for RelayChainCli {
	fn p2p_listen_port() -> u16 {
		30334
	}

	fn rpc_ws_listen_port() -> u16 {
		9945
	}

	fn rpc_http_listen_port() -> u16 {
		9934
	}

	fn prometheus_listen_port() -> u16 {
		9616
	}
}

impl CliConfiguration<Self> for RelayChainCli {
	fn shared_params(&self) -> &SharedParams {
		self.base.base.shared_params()
	}

	fn import_params(&self) -> Option<&ImportParams> {
		self.base.base.import_params()
	}

	fn network_params(&self) -> Option<&NetworkParams> {
		self.base.base.network_params()
	}

	fn keystore_params(&self) -> Option<&KeystoreParams> {
		self.base.base.keystore_params()
	}

	fn base_path(&self) -> Result<Option<BasePath>> {
		Ok(self
			.shared_params()
			.base_path()?
			.or_else(|| self.base_path.clone().map(Into::into)))
	}

	fn rpc_http(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
		self.base.base.rpc_http(default_listen_port)
	}

	fn rpc_ipc(&self) -> Result<Option<String>> {
		self.base.base.rpc_ipc()
	}

	fn rpc_ws(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
		self.base.base.rpc_ws(default_listen_port)
	}

	fn prometheus_config(
		&self,
		default_listen_port: u16,
		chain_spec: &Box<dyn ChainSpec>,
	) -> Result<Option<PrometheusConfig>> {
		self.base.base.prometheus_config(default_listen_port, chain_spec)
	}

	fn init<F>(
		&self,
		_support_url: &String,
		_impl_version: &String,
		_logger_hook: F,
		_config: &sc_service::Configuration,
	) -> Result<()>
	where
		F: FnOnce(&mut sc_cli::LoggerBuilder, &sc_service::Configuration),
	{
		unreachable!("PolkadotCli is never initialized; qed");
	}

	fn chain_id(&self, is_dev: bool) -> Result<String> {
		let chain_id = self.base.base.chain_id(is_dev)?;

		Ok(if chain_id.is_empty() { self.chain_id.clone().unwrap_or_default() } else { chain_id })
	}

	fn role(&self, is_dev: bool) -> Result<sc_service::Role> {
		self.base.base.role(is_dev)
	}

	fn transaction_pool(&self, is_dev: bool) -> Result<sc_service::config::TransactionPoolOptions> {
		self.base.base.transaction_pool(is_dev)
	}

	fn trie_cache_maximum_size(&self) -> Result<Option<usize>> {
		self.base.base.trie_cache_maximum_size()
	}

	fn rpc_methods(&self) -> Result<sc_service::config::RpcMethods> {
		self.base.base.rpc_methods()
	}

	fn rpc_ws_max_connections(&self) -> Result<Option<usize>> {
		self.base.base.rpc_ws_max_connections()
	}

	fn rpc_cors(&self, is_dev: bool) -> Result<Option<Vec<String>>> {
		self.base.base.rpc_cors(is_dev)
	}

	fn default_heap_pages(&self) -> Result<Option<u64>> {
		self.base.base.default_heap_pages()
	}

	fn force_authoring(&self) -> Result<bool> {
		self.base.base.force_authoring()
	}

	fn disable_grandpa(&self) -> Result<bool> {
		self.base.base.disable_grandpa()
	}

	fn max_runtime_instances(&self) -> Result<Option<usize>> {
		self.base.base.max_runtime_instances()
	}

	fn announce_block(&self) -> Result<bool> {
		self.base.base.announce_block()
	}

	fn telemetry_endpoints(
		&self,
		chain_spec: &Box<dyn ChainSpec>,
	) -> Result<Option<sc_telemetry::TelemetryEndpoints>> {
		self.base.base.telemetry_endpoints(chain_spec)
	}

	fn node_name(&self) -> Result<String> {
		self.base.base.node_name()
	}
}
