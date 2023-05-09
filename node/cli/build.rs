use substrate_build_script_utils::{generate_cargo_keys, rerun_if_git_head_changed};

// Don't allow both frequency and all-frequency-features (except for lint/check) so that we always have a good mainnet runtime
#[cfg(all(
	not(feature = "frequency-lint-check"),
	feature = "all-frequency-features",
	any(
		feature = "frequency",
		feature = "frequency-no-relay",
		feature = "frequency-rococo-local",
		feature = "frequency-rococo-testnet"
	)
))]
compile_error!("\"all-frequency-features\" cannot be enabled at the same time as another main feature except for a lint/check with \"frequency-lint-check\"");

fn main() {
	generate_cargo_keys();

	rerun_if_git_head_changed();
}
