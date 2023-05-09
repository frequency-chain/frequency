use substrate_build_script_utils::{generate_cargo_keys, rerun_if_git_head_changed};

// Don't allow more than one main feature (except for benchmark/lint/check) so that we always have a good mainnet runtime
#[cfg(all(
	not(feature = "frequency-lint-check"),
	feature = "frequency",
	any(
		feature = "frequency-no-relay",
		feature = "frequency-rococo-local",
		feature = "frequency-rococo-testnet"
	)
))]
compile_error!("\"Only one main feature can be enabled except for benchmark/lint/check with \"frequency-lint-check\"");

// Don't allow more than one main feature (except for benchmark/lint/check) so that we always have a good mainnet runtime
#[cfg(all(
	not(feature = "frequency-lint-check"),
	feature = "frequency-no-relay",
	any(
		feature = "frequency",
		feature = "frequency-rococo-local",
		feature = "frequency-rococo-testnet"
	)
))]
compile_error!("\"Only one main feature can be enabled except for benchmark/lint/check with \"frequency-lint-check\"");

// Don't allow more than one main feature (except for benchmark/lint/check) so that we always have a good mainnet runtime
#[cfg(all(
	not(feature = "frequency-lint-check"),
	feature = "frequency-rococo-local",
	any(
		feature = "frequency",
		feature = "frequency-no-relay",
		feature = "frequency-rococo-testnet"
	)
))]
compile_error!("\"Only one main feature can be enabled except for benchmark/lint/check with \"frequency-lint-check\"");

// Don't allow more than one main feature (except for benchmark/lint/check) so that we always have a good mainnet runtime
#[cfg(all(
	not(feature = "frequency-lint-check"),
	feature = "frequency-rococo-testnet",
	any(feature = "frequency", feature = "frequency-no-relay", feature = "frequency-rococo-local",)
))]
compile_error!("\"Only one main feature can be enabled except for benchmark/lint/check with \"frequency-lint-check\"");

fn main() {
	generate_cargo_keys();

	rerun_if_git_head_changed();
}
