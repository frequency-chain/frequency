use crate::cli::Cli;
use frequency_service::block_sealing::start_frequency_dev_sealing_node;
use sc_cli::SubstrateCli;

#[allow(clippy::result_large_err)]
pub fn run_as_localchain(cli: Cli) -> sc_service::Result<(), sc_cli::Error> {
	let runner = cli.create_runner(&cli.run.normalize())?;
	runner.run_node_until_exit(|config| async move {
		start_frequency_dev_sealing_node(
			config,
			cli.sealing,
			u16::from(cli.sealing_interval),
			cli.sealing_create_empty_blocks,
		)
		.map_err(Into::into)
	})
}
