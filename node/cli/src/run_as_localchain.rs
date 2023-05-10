use crate::cli::{Cli, SealingMode};
use sc_cli::SubstrateCli;

pub fn run_as_localchain(cli: Cli) -> sc_service::Result<(), sc_cli::Error> {
	let runner = cli.create_runner(&cli.run.normalize())?;
	runner.run_node_until_exit(|config| async move {
		match cli.sealing {
			SealingMode::Instant =>
				return frequency_service::service::frequency_dev_instant_sealing(config, true)
					.map_err(Into::into),
			SealingMode::Manual =>
				return frequency_service::service::frequency_dev_instant_sealing(config, false)
					.map_err(Into::into),
			SealingMode::Interval =>
				return frequency_service::service::frequency_dev_instant_sealing(config, true)
					.map_err(Into::into),
		}
	})
}
