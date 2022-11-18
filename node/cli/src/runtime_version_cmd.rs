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
	pub async fn run(&self) -> Result<(), Error> {
		let runtime_version: RuntimeVersion = self.read_runtime_version().unwrap();
		let result = json!(runtime_version);
		let file: Box<dyn io::Write> = match &self.output {
			Some(filename) => Box::new(fs::File::create(filename)?),
			None => Box::new(io::stdout()),
		};
		to_writer(file, &result).map_err(|_| Error::from("Failed Encoding"))
	}

	#[allow(unreachable_code)]
	pub fn read_runtime_version(&self) -> Result<RuntimeVersion, Error> {
		#[cfg(feature = "frequency")]
		{
			return Ok(frequency_service::service::frequency_runtime::VERSION)
		}
		#[cfg(any(feature = "frequency-rococo-testnet", feature = "frequency-rococo-local"))]
		{
			return Ok(frequency_service::service::frequency_rococo_runtime::VERSION)
		}
		return Err(Error::from("No runtime version available"))
	}
}

impl CliConfiguration for ExportRuntimeVersionCmd {
	fn shared_params(&self) -> &SharedParams {
		&self.shared_params
	}
}
