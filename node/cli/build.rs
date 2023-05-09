use substrate_build_script_utils::{generate_cargo_keys, rerun_if_git_head_changed};

// Don't allow both frequency and all-frequency-features (except for lint/check) so that we always have a good mainnet runtime
#[cfg(all(
	feature = "frequency",
	feature = "all-frequency-features",
	not(feature = "frequency-lint-check")
))]
compile_error!("feature \"frequency\" and \"all-frequency-features\" cannot be enabled at the same time except for lint/check with \"frequency-lint-check\"");

// Don't allow both frequency-no-relay and all-frequency-features (except for lint/check) so that we always have a good mainnet runtime
#[cfg(all(
	feature = "frequency-no-relay",
	feature = "all-frequency-features",
	not(feature = "frequency-lint-check")
))]
compile_error!("feature \"frequency-no-relay\" and \"all-frequency-features\" cannot be enabled at the same time except for lint/check with \"frequency-lint-check\"");

// Don't allow both frequency-rococo-local and all-frequency-features (except for lint/check) so that we always have a good mainnet runtime
#[cfg(all(
	feature = "frequency-rococo-local",
	feature = "all-frequency-features",
	not(feature = "frequency-lint-check")
))]
compile_error!("feature \"frequency-rococo-local\" and \"all-frequency-features\" cannot be enabled at the same time except for lint/check with \"frequency-lint-check\"");

// Don't allow both frequency-rococo-testnet and all-frequency-features (except for lint/check) so that we always have a good mainnet runtime
#[cfg(all(
	feature = "frequency-rococo-testnet",
	feature = "all-frequency-features",
	not(feature = "frequency-lint-check")
))]
compile_error!("feature \"frequency-rococo-testnet\" and \"all-frequency-features\" cannot be enabled at the same time except for lint/check with \"frequency-lint-check\"");

fn main() {
	generate_cargo_keys();

	rerun_if_git_head_changed();
}
