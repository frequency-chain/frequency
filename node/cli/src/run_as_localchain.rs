use crate::cli::Cli;
use frequency_service::block_sealing::frequency_dev_sealing;
use sc_cli::SubstrateCli;
use std::process;

pub fn run_as_localchain(cli: Cli) -> sc_service::Result<(), sc_cli::Error> {
	let runner = cli.create_runner(&cli.run.normalize())?;

	runner.run_node_until_exit(|config| async move {
		if cli.instant_sealing {
			log::error!(
				"The --instant-sealing option is deprecated.  Use --sealing=instant instead."
			);
			process::exit(1);
		} else if cli.manual_sealing {
			log::error!(
				"The --manual-sealing option is deprecated.  Use --sealing=manual instead."
			);
			process::exit(1);
		}
		frequency_dev_sealing(
			config,
			cli.sealing,
			u16::from(cli.sealing_interval),
			cli.sealing_allow_empty_blocks,
		)
		.map_err(Into::into)
	})
}
