use clap::Parser;
use polkadot_cli::ProvideRuntimeApi;
use sc_cli::{CliConfiguration, Error, GenericNumber, SharedParams};
use serde_json::{json, to_writer};
use sp_api::Metadata;
use sp_core::Bytes;
use sp_runtime::{
	generic::BlockId,
	traits::{Block as BlockT, Header as HeaderT},
};
use std::{fmt::Debug, fs, io, path::PathBuf, str::FromStr, sync::Arc};

/// The `export-metadata` command used to export chain metadata.
#[derive(Debug, Clone, Parser)]
pub struct ExportOpenApiCmd {
	/// Output file name or stdout if unspecified.
	#[clap(value_parser)]
	pub output: Option<PathBuf>,

	/// Specify starting block number.
	///
	/// Default is 0.
	#[clap(long, value_name = "BLOCK")]
	pub from: Option<GenericNumber>,

	#[allow(missing_docs)]
	#[clap(flatten)]
	pub shared_params: SharedParams,
}

impl ExportOpenApiCmd {
	/// Run the export-metadata command
	pub async fn run<B, C>(&self, client: Arc<C>) -> Result<(), Error>
	where
		B: BlockT,
		C: ProvideRuntimeApi<B>,
		C::Api: sp_api::Metadata<B> + 'static,
		<<B::Header as HeaderT>::Number as FromStr>::Err: Debug,
	{
		Ok(())
	}
}

impl CliConfiguration for ExportOpenApiCmd {
	fn shared_params(&self) -> &SharedParams {
		&self.shared_params
	}
}
