use clap::Parser;
use sc_cli::{CliConfiguration, Error, RuntimeVersion, SharedParams};
use serde_json::{json, to_writer};
use std::{fmt::Debug, fs, io, path::PathBuf};

/// The `export-metadata` command used to export chain metadata.
#[derive(Debug, Clone, Parser)]
pub struct ExportRuntimeVersionCmd {
	/// Output file name or stdout if unspecified.
	#[clap(value_parser)]
	pub output: Option<PathBuf>,

	#[allow(missing_docs)]
	#[clap(flatten)]
	pub shared_params: SharedParams,
}

impl ExportRuntimeVersionCmd {
	/// Run the get-runtime-version command.
	pub async fn run(&self, runtime_version: &RuntimeVersion) -> Result<(), Error> {
		let result = json!(runtime_version);
		let file: Box<dyn io::Write> = match &self.output {
			Some(filename) => Box::new(fs::File::create(filename)?),
			None => Box::new(io::stdout()),
		};
		to_writer(file, &result)
			.map_err(|_| Error::from("export-runtime-version: failed encoding"))?;
	}
}

impl CliConfiguration for ExportRuntimeVersionCmd {
	fn shared_params(&self) -> &SharedParams {
		&self.shared_params
	}
}
