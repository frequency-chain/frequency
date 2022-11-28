#![cfg_attr(not(feature = "std"), no_std)]
pub mod constants;
pub mod extensions;
pub mod fee;
pub mod weights;

/// Macro to set a value (e.g. when using the `parameter_types` macro) to either a production value
/// or to an environment variable or testing value (in case the `frequency-rococo-local` feature is selected or in instant-sealing mode).
/// Note that the environment variable is evaluated _at compile time_.
///
/// Usage:
/// ```Rust
/// parameter_types! {
/// 	// Note that the env variable version parameter cannot be const.
/// 	pub LaunchPeriod: BlockNumber = prod_or_testnet_or_local_or_env!(7 * DAYS, 28 * DAYS, 1 * MINUTES, "FRQCY_LAUNCH_PERIOD");
/// 	pub const VotingPeriod: BlockNumber = prod_or_testnet_or_local_or_env!(7 * DAYS, 28 * DAYS, 1 * MINUTES);
/// }
/// ```
#[macro_export]
macro_rules! prod_or_testnet_or_local_or_env {
	($prod:expr, $test:expr, $local:expr) => {
		if cfg!(feature = "frequency-rococo-local") {
			$local
		} else if cfg!(feature = "frequency-rococo-testnet") {
			$test
		} else {
			$prod
		}
	};
	($prod:expr, $test:expr, $local:expr, $env:expr) => {
		if cfg!(feature = "frequency-rococo-local") {
			core::option_env!($env).map(|s| s.parse().ok()).flatten().unwrap_or($local)
		} else if cfg!(feature = "frequency-rococo-testnet") {
			$test
		} else {
			$prod
		}
	};
}

/// Macro to set a value (e.g. when using the `parameter_types` macro) to either a production value
/// or to an environment variable or testing value (in case the `frequency-rococo` feature is selected or in instant-sealing mode).
/// Note that the environment variable is evaluated _at compile time_.
#[macro_export]
macro_rules! create_runtime_str_for_network {
	($network:expr) => {
		if cfg!(feature = "frequency-rococo-local") || cfg!(feature = "frequency-rococo-testnet") {
			sp_version::create_runtime_str!(concat!($network, "-rococo"))
		} else {
			sp_version::create_runtime_str!($network)
		}
	};
}
