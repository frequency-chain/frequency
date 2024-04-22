use substrate_build_script_utils::{generate_cargo_keys, rerun_if_git_head_changed};

#[cfg(not(any(
	feature = "frequency",
	feature = "frequency-local",
	feature = "frequency-no-relay",
	feature = "frequency-testnet"
)))]
compile_error!(
	r#"You must enable one of these features:
- Mainnet: "frequency"
- Frequency Rococo: "frequency-testnet"
- Local: "frequency-local"
- No Relay: "frequency-no-relay",
- All: "frequency-lint-check"#
);

// Don't allow more than one main feature (except for benchmark/lint/check) so that we always have a good mainnet runtime
#[cfg(all(
	not(feature = "frequency-lint-check"),
	feature = "frequency",
	any(
		feature = "frequency-no-relay",
		feature = "frequency-local",
		feature = "frequency-testnet"
	)
))]
compile_error!("\"Only one main feature can be enabled except for benchmark/lint/check with \"frequency-lint-check\"");

#[cfg(all(
	not(feature = "frequency-lint-check"),
	feature = "frequency-no-relay",
	any(feature = "frequency", feature = "frequency-local", feature = "frequency-testnet")
))]
compile_error!("\"Only one main feature can be enabled except for benchmark/lint/check with \"frequency-lint-check\"");

#[cfg(all(
	not(feature = "frequency-lint-check"),
	feature = "frequency-local",
	any(feature = "frequency", feature = "frequency-no-relay", feature = "frequency-testnet")
))]
compile_error!("\"Only one main feature can be enabled except for benchmark/lint/check with \"frequency-lint-check\"");

#[cfg(all(
	not(feature = "frequency-lint-check"),
	feature = "frequency-testnet",
	any(feature = "frequency", feature = "frequency-no-relay", feature = "frequency-local",)
))]
compile_error!("\"Only one main feature can be enabled except for benchmark/lint/check with \"frequency-lint-check\"");

fn main() {
	generate_cargo_keys();

	rerun_if_git_head_changed();
}
