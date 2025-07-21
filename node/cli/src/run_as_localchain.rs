use crate::cli::Cli;
use frequency_service::block_sealing::start_frequency_dev_sealing_node;
use polkadot_service::TransactionPoolOptions;
use sc_cli::{SubstrateCli, TransactionPoolType};

pub fn run_as_localchain(cli: Cli) -> sc_service::Result<(), sc_cli::Error> {
	let runner = cli.create_runner(&cli.run.normalize())?;
	let override_pool_config = TransactionPoolOptions::new_with_params(
		cli.run.base.pool_config.pool_limit,
		cli.run.base.pool_config.pool_kbytes * 1024,
		cli.run.base.pool_config.tx_ban_seconds,
		TransactionPoolType::SingleState.into(),
		true,
	);

	runner.run_node_until_exit(|config| async move {
		start_frequency_dev_sealing_node(
			config,
			cli.sealing,
			u16::from(cli.sealing_interval),
			cli.sealing_create_empty_blocks,
			Some(override_pool_config),
		)
		.map_err(Into::into)
	})
}
