use clap::Parser;
use sc_cli::{CliConfiguration, Error, GenericNumber, SharedParams};
use serde_json::{json, to_writer};
use sp_api::{Metadata, ProvideRuntimeApi};
use sp_core::Bytes;
use sp_runtime::{
	traits::{Block as BlockT, Header as HeaderT},
};
use std::{fmt::Debug, fs, io, path::PathBuf, str::FromStr, sync::Arc};
use sc_client_api::HeaderBackend;

/// The `export-metadata` command used to export chain metadata.
/// Remember that this uses the chain database. So it will pull the _current_ metadata from that database.
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

	/// Use a temporary directory for the db
	///
	/// A temporary directory will be created to store the configuration and will be deleted
	/// at the end of the process.
	///
	/// Note: the directory is random per process execution. This directory is used as base path
	/// which includes: database, node key and keystore.
	#[arg(long, conflicts_with = "base_path")]
	pub tmp: bool,
}

#[allow(clippy::unwrap_used)]
impl ExportMetadataCmd {
	/// Run the export-metadata command
	pub async fn run<B, C>(&self, client: Arc<C>) -> Result<(), Error>
	where
		B: BlockT,
		C: ProvideRuntimeApi<B> + HeaderBackend<B>,
		C::Api: Metadata<B> + 'static,
		<<B::Header as HeaderT>::Number as FromStr>::Err: Debug,
	{
		let api = client.runtime_api();

		let block_number = self.from.as_ref().and_then(|f| f.parse().ok()).unwrap_or(0u32);
		let maybe_hash = client.hash(block_number.into())?;
		let block_hash = maybe_hash.ok_or_else(|| Error::from("Block not found"))?;

		let metadata: Bytes =
			api.metadata(block_hash).unwrap().into();
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

	// Enabling `--tmp` on this command
	fn base_path(&self) -> Result<Option<sc_service::BasePath>, sc_cli::Error> {
		match &self.tmp {
			true => Ok(Some(sc_service::BasePath::new_temp_dir()?)),
			false => self.shared_params.base_path(),
		}
	}
}
