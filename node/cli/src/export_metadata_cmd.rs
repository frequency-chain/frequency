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
pub struct ExportMetadataCmd {
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

impl ExportMetadataCmd {
	/// Run the export-metadata command
	pub async fn run<B, C>(&self, client: Arc<C>) -> Result<(), Error>
	where
		B: BlockT,
		C: ProvideRuntimeApi<B>,
		C::Api: sp_api::Metadata<B> + 'static,
		<<B::Header as HeaderT>::Number as FromStr>::Err: Debug,
	{
		let from = self.from.as_ref().and_then(|f| f.parse().ok()).unwrap_or(0u32);
		let metadata: Bytes =
			client.runtime_api().metadata(&BlockId::number(from.into())).unwrap().into();
		let result = json!({ "result": metadata });

		let file: Box<dyn io::Write> = match &self.output {
			Some(filename) => Box::new(fs::File::create(filename)?),
			None => Box::new(io::stdout()),
		};
		to_writer(file, &result).map_err(|_| Error::from("Failed Encoding"))
	}
}

impl CliConfiguration for ExportMetadataCmd {
	fn shared_params(&self) -> &SharedParams {
		&self.shared_params
	}
}
