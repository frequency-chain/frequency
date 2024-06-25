//! Frequency CLI library.

// File originally from https://github.com/paritytech/cumulus/blob/master/parachain-template/node/src/cli.rs

use crate::{ExportMetadataCmd, ExportRuntimeVersionCmd};
use std::path::PathBuf;

#[cfg(feature = "frequency-no-relay")]
use std::num::NonZeroU16;

#[cfg(feature = "frequency-no-relay")]
use cli_opt::SealingMode;

/// Sub-commands supported by the collator.
#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
	/// Build a chain specification.
	BuildSpec(sc_cli::BuildSpecCmd),

	/// Validate blocks.
	CheckBlock(sc_cli::CheckBlockCmd),

	/// Export blocks.
	ExportBlocks(sc_cli::ExportBlocksCmd),

	/// Export the state of a given block into a chain spec.
	ExportState(sc_cli::ExportStateCmd),

	/// Import blocks.
	ImportBlocks(sc_cli::ImportBlocksCmd),

	/// Export metadata.
	ExportMetadata(ExportMetadataCmd),

	/// Revert the chain to a previous state.
	Revert(sc_cli::RevertCmd),

	/// Remove the whole chain.
	PurgeChain(cumulus_client_cli::PurgeChainCmd),

	/// Export the genesis state of the parachain.
	ExportGenesisState(cumulus_client_cli::ExportGenesisStateCommand),

	/// Export the genesis wasm of the parachain.
	ExportGenesisWasm(cumulus_client_cli::ExportGenesisWasmCommand),

	/// Sub-commands concerned with benchmarking.
	/// The pallet benchmarking moved to the `pallet` sub-command.
	#[clap(subcommand)]
	Benchmark(frame_benchmarking_cli::BenchmarkCmd),

	/// Get current runtime spec version.
	ExportRuntimeVersion(ExportRuntimeVersionCmd),
}

#[derive(Debug, clap::Parser)]
#[clap(
	propagate_version = true,
	args_conflicts_with_subcommands = true,
	subcommand_negates_reqs = true
)]
pub struct Cli {
	#[clap(subcommand)]
	pub subcommand: Option<Subcommand>,

	#[clap(flatten)]
	pub run: cumulus_client_cli::RunCmd,

	/// Disable automatic hardware benchmarks.
	///
	/// By default these benchmarks are automatically ran at startup and measure
	/// the CPU speed, the memory bandwidth and the disk speed.
	///
	/// The results are then printed out in the logs, and also sent as part of
	/// telemetry, if telemetry is enabled.
	#[clap(long)]
	pub no_hardware_benchmarks: bool,

	/// Relay chain arguments
	#[clap(raw = true)]
	pub relay_chain_args: Vec<String>,

	/// Instant block sealing
	/// Blocks are triggered to be formed each time a transaction hits the validated transaction pool
	/// Empty blocks can also be formed using the `engine_createBlock` RPC
	#[cfg(feature = "frequency-no-relay")]
	#[clap(long, value_enum, help = "The sealing mode", default_value_t=SealingMode::Instant)]
	pub sealing: SealingMode,

	/// Interval in seconds for interval sealing.
	#[cfg(feature = "frequency-no-relay")]
	#[clap(long, help = "The interval in seconds", default_value = "12", value_name = "SECONDS")]
	pub sealing_interval: NonZeroU16,

	/// Whether to create empty blocks in manual and interval sealing modes.
	#[cfg(feature = "frequency-no-relay")]
	#[clap(long, help = "Create empty blocks in interval sealing modes", default_value = "false")]
	pub sealing_create_empty_blocks: bool,
}

#[derive(Debug)]
pub struct RelayChainCli {
	/// The actual relay chain cli object.
	pub base: polkadot_cli::RunCmd,

	/// Optional chain id that should be passed to the relay chain.
	pub chain_id: Option<String>,

	/// The base path that should be used by the relay chain.
	pub base_path: Option<PathBuf>,
}

impl RelayChainCli {
	/// Parse the relay chain CLI parameters using the para chain `Configuration`.
	pub fn new<'a>(
		para_config: &sc_service::Configuration,
		relay_chain_args: impl Iterator<Item = &'a String>,
	) -> Self {
		let extension =
			frequency_service::chain_spec::Extensions::try_get(&*para_config.chain_spec);
		let chain_id = extension.map(|e| e.relay_chain.clone());
		let base_path = para_config.base_path.path().join("polkadot");
		Self {
			base_path: Some(base_path),
			chain_id,
			base: clap::Parser::parse_from(relay_chain_args),
		}
	}
}
