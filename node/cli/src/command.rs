// File originally from https://github.com/paritytech/cumulus/blob/master/parachain-template/node/src/command.rs

use crate::{
	benchmarking::{inherent_benchmark_data, RemarkBuilder},
	cli::{Cli, RelayChainCli, Subcommand},
};
use common_primitives::node::Block;
use cumulus_client_service::storage_proof_size::HostFunctions as ReclaimHostFunctions;
use frame_benchmarking_cli::BenchmarkCmd;
use frequency_service::{
	chain_spec,
	service::{frequency_runtime::VERSION, new_partial},
};
use sc_cli::{
	ChainSpec, CliConfiguration, DefaultConfigurationValues, ImportParams, KeystoreParams,
	NetworkParams, Result, RuntimeVersion, SharedParams, SubstrateCli,
};
use sc_service::config::{BasePath, PrometheusConfig};
use sp_runtime::traits::HashingFor;

enum ChainIdentity {
	Frequency,
	FrequencyPaseo,
	FrequencyLocal,
	FrequencyDev,
}

trait IdentifyChain {
	fn identify(&self) -> ChainIdentity;
}

impl IdentifyChain for dyn sc_service::ChainSpec {
	fn identify(&self) -> ChainIdentity {
		if self.id() == "frequency" {
			ChainIdentity::Frequency
		} else if self.id() == "frequency-paseo" {
			ChainIdentity::FrequencyPaseo
		} else if self.id() == "frequency-local" {
			ChainIdentity::FrequencyLocal
		} else if self.id() == "dev" {
			ChainIdentity::FrequencyDev
		} else {
			panic!("Unknown chain identity")
		}
	}
}

impl PartialEq for ChainIdentity {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(ChainIdentity::Frequency, ChainIdentity::Frequency) => true,
			(ChainIdentity::FrequencyPaseo, ChainIdentity::FrequencyPaseo) => true,
			(ChainIdentity::FrequencyLocal, ChainIdentity::FrequencyLocal) => true,
			(ChainIdentity::FrequencyDev, ChainIdentity::FrequencyDev) => true,
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
		#[cfg(feature = "frequency-no-relay")]
		"dev" | "frequency-no-relay" =>
			return Ok(Box::new(chain_spec::frequency_dev::development_config())),
		#[cfg(feature = "frequency-local")]
		"frequency-paseo-local" =>
			return Ok(Box::new(chain_spec::frequency_paseo::local_paseo_testnet_config())),
		#[cfg(feature = "frequency-testnet")]
		"frequency-testnet" | "frequency-paseo" | "paseo" | "testnet" =>
			return Ok(Box::new(chain_spec::frequency_paseo::load_frequency_paseo_spec())),
		path => {
			if path.is_empty() {
				if cfg!(feature = "frequency") {
					#[cfg(feature = "frequency")]
					{
						return Ok(Box::new(chain_spec::frequency::load_frequency_spec()));
					}
					#[cfg(not(feature = "frequency"))]
					return Err("Frequency runtime is not available.".into());
				} else if cfg!(feature = "frequency-no-relay") {
					#[cfg(feature = "frequency-no-relay")]
					{
						return Ok(Box::new(chain_spec::frequency_dev::development_config()));
					}
					#[cfg(not(feature = "frequency-no-relay"))]
					return Err("Frequency Development (no relay) runtime is not available.".into());
				} else if cfg!(feature = "frequency-local") {
					#[cfg(feature = "frequency-local")]
					{
						return Ok(Box::new(
							chain_spec::frequency_paseo::local_paseo_testnet_config(),
						));
					}
					#[cfg(not(feature = "frequency-local"))]
					return Err("Frequency Local runtime is not available.".into());
				} else if cfg!(feature = "frequency-testnet") {
					#[cfg(feature = "frequency-testnet")]
					{
						return Ok(Box::new(
							chain_spec::frequency_paseo::load_frequency_paseo_spec(),
						));
					}
					#[cfg(not(feature = "frequency-testnet"))]
					return Err("Frequency Paseo runtime is not available.".into());
				} else {
					return Err("No chain spec is available.".into());
				}
			}
			let path_buf = std::path::PathBuf::from(path);
			let spec = Box::new(chain_spec::DummyChainSpec::from_json_file(path_buf.clone())?)
				as Box<dyn ChainSpec>;
			if ChainIdentity::Frequency == spec.identify() {
				#[cfg(feature = "frequency")]
				{
					return Ok(Box::new(chain_spec::frequency::ChainSpec::from_json_file(
						path_buf,
					)?));
				}
				#[cfg(not(feature = "frequency"))]
				return Err("Frequency runtime is not available.".into());
			} else if ChainIdentity::FrequencyPaseo == spec.identify() {
				#[cfg(feature = "frequency-testnet")]
				{
					return Ok(Box::new(chain_spec::frequency_paseo::ChainSpec::from_json_file(
						path_buf,
					)?));
				}
				#[cfg(not(feature = "frequency-testnet"))]
				return Err("Frequency Paseo runtime is not available.".into());
			} else if ChainIdentity::FrequencyLocal == spec.identify() {
				#[cfg(feature = "frequency-local")]
				{
					return Ok(Box::new(chain_spec::frequency_paseo::ChainSpec::from_json_file(
						path_buf,
					)?));
				}
				#[cfg(not(feature = "frequency-local"))]
				return Err("Frequency Local runtime is not available.".into());
			} else if ChainIdentity::FrequencyDev == spec.identify() {
				#[cfg(feature = "frequency-no-relay")]
				{
					return Ok(Box::new(chain_spec::frequency_paseo::ChainSpec::from_json_file(
						path_buf,
					)?));
				}
				#[cfg(not(feature = "frequency-no-relay"))]
				return Err("Frequency Dev (no relay) runtime is not available.".into());
			} else {
				return Err("Unknown chain spec.".into());
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
		"https://github.com/frequency-chain/frequency/issues/new".into()
	}

	fn copyright_start_year() -> i32 {
		2020
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		load_spec(id)
	}
}

impl Cli {
	/// Returns a reference to the runtime version.
	fn runtime_version() -> &'static RuntimeVersion {
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
		match id {
			// TODO: Remove once on a Polkadot-SDK with Paseo-Local
			#[cfg(feature = "frequency-local")]
			"paseo-local" => return Ok(Box::new(chain_spec::frequency_paseo::load_paseo_local_spec())),
			// TODO: Remove once on a Polkadot-SDK with Paseo
			#[cfg(feature = "frequency-testnet")]
			"paseo" => return Ok(Box::new(chain_spec::frequency_paseo::load_paseo_spec())),
			_ =>
				return polkadot_cli::Cli::from_iter([RelayChainCli::executable_name()].iter())
					.load_spec(id),
		}
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
		Some(Subcommand::ExportGenesisHead(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| {
				let partials = new_partial(&config, false)?;

				cmd.run(partials.client)
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
						runner.sync_run(|config| {
							cmd.run::<HashingFor<Block>, ReclaimHostFunctions>(config)
						})
					} else {
						return Err("Benchmarking wasn't enabled when building the node. \
									You can enable it with `--features runtime-benchmarks`."
							.into());
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

		Some(Subcommand::ExportRuntimeVersion(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			runner.async_run(|config| {
				let version = Cli::runtime_version();
				// grab the task manager.
				let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
				let task_manager =
					sc_service::TaskManager::new(config.tokio_handle.clone(), registry)
						.map_err(|e| format!("Error: {:?}", e))?;
				Ok((cmd.run(version), task_manager))
			})
		},
		None => run_chain(cli),
	}
}

// This appears messy but due to layers of Rust complexity, it's necessary.
pub fn run_chain(cli: Cli) -> sc_service::Result<(), sc_cli::Error> {
	#[allow(unused)]
	let mut result: sc_service::Result<(), polkadot_cli::Error> = Ok(());
	#[cfg(feature = "frequency-no-relay")]
	{
		result = crate::run_as_localchain::run_as_localchain(cli);
	}
	#[cfg(not(feature = "frequency-no-relay"))]
	{
		result = crate::run_as_parachain::run_as_parachain(cli);
	}

	result
}

impl DefaultConfigurationValues for RelayChainCli {
	fn p2p_listen_port() -> u16 {
		30334
	}

	fn rpc_listen_port() -> u16 {
		9945
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

	fn rpc_cors(&self, is_dev: bool) -> Result<Option<Vec<String>>> {
		self.base.base.rpc_cors(is_dev)
	}

	fn default_heap_pages(&self) -> Result<Option<u64>> {
		self.base.base.default_heap_pages()
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
