use crate::cli::{Cli, RelayChainCli};
use cumulus_primitives_core::ParaId;
use frame_benchmarking_cli::SUBSTRATE_REFERENCE_HARDWARE;
use frequency_service::chain_spec;
use log::info;
use polkadot_service::TransactionPoolOptions;
use sc_cli::{SubstrateCli, TransactionPoolType};

pub fn run_as_parachain(cli: Cli) -> sc_service::Result<(), sc_cli::Error> {
	let runner = cli.create_runner(&cli.run.normalize())?;
	let collator_options = cli.run.collator_options();
	let override_pool_config = TransactionPoolOptions::new_with_params(
		cli.run.base.pool_config.pool_limit,
		cli.run.base.pool_config.pool_kbytes * 1024,
		cli.run.base.pool_config.tx_ban_seconds,
		TransactionPoolType::ForkAware.into(),
		false,
	);

	runner.run_node_until_exit(|config| async move {
		let hwbench = (!cli.no_hardware_benchmarks)
			.then_some(config.database.path().map(|database_path| {
				let _ = std::fs::create_dir_all(database_path);
				sc_sysinfo::gather_hwbench(Some(database_path), &SUBSTRATE_REFERENCE_HARDWARE)
			}))
			.flatten();

		let para_id = chain_spec::Extensions::try_get(&*config.chain_spec)
			.map(|e: &chain_spec::Extensions| e.para_id)
			.ok_or("Could not find parachain ID in chain-spec.")?;

		let polkadot_cli = RelayChainCli::new(
			&config,
			[RelayChainCli::executable_name()].iter().chain(cli.relay_chain_args.iter()),
		);

		let id = ParaId::from(para_id);

		let tokio_handle = config.tokio_handle.clone();

		let polkadot_config =
			SubstrateCli::create_configuration(&polkadot_cli, &polkadot_cli, tokio_handle)
				.map_err(|err| format!("Relay chain argument error: {}", err))?;

		info!("Parachain id: {:?}", id);
		info!("Is collating: {}", if config.role.is_authority() { "yes" } else { "no" });

		frequency_service::service::start_parachain_node(
			config,
			polkadot_config,
			collator_options,
			id,
			hwbench,
			Some(override_pool_config),
		)
		.await
		.map(|r| r.0)
		.map_err(Into::into)
	})
}
