use clap::Parser;
use pallet_messages_rpc::open_api::MessagesOAIHandler;
use sc_cli::{CliConfiguration, Error, GenericNumber, SharedParams};
use serde_json::to_writer;
use std::{fmt::Debug, fs, io, path::PathBuf};

/// The `export-metadata` command used to export chain metadata.
#[derive(Debug, Clone, Parser)]
pub struct ExportOpenApiCmd {
	/// Output file name or stdout if unspecified.
	#[clap(value_parser)]
	pub output: Option<PathBuf>,

	#[allow(missing_docs)]
	#[clap(flatten)]
	pub shared_params: SharedParams,
}

impl ExportOpenApiCmd {
	/// Run the export-open-api command
	pub async fn run(&self) -> Result<(), Error> {
		let messages_spec = MessagesOAIHandler::get_open_api().await.unwrap_or_default();
		let file: Box<dyn io::Write> = match &self.output {
			Some(filename) => Box::new(fs::File::create(filename)?),
			None => Box::new(io::stdout()),
		};
		to_writer(file, &messages_spec).map_err(|_| Error::from("Failed Encoding"))
	}
}

impl CliConfiguration for ExportOpenApiCmd {
	fn shared_params(&self) -> &SharedParams {
		&self.shared_params
	}
}
