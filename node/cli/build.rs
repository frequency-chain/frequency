use substrate_build_script_utils::{generate_cargo_keys, rerun_if_git_head_changed};

// // Don't allow both frequency and all-frequency-features so that we always have a good mainnet runtime
#[cfg(all(
	feature = "frequency",
	feature = "all-frequency-features",
	not(feature = "runtime-benchmarks")
))]
compile_error!("feature \"frequency\" and feature \"all-frequency-features\" cannot be enabled at the same time");

// // Don't allow both frequency-no-relay and all-frequency-features so that we always have a good mainnet runtime
#[cfg(all(
	feature = "frequency",
	feature = "all-frequency-features",
	not(feature = "runtime-benchmarks")
))]
compile_error!("feature \"frequency-no-relay\" and feature \"all-frequency-features\" cannot be enabled at the same time");

fn main() {
	generate_cargo_keys();

	rerun_if_git_head_changed();
}
