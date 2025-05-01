#![cfg_attr(not(feature = "std"), no_std)]

pub mod constants;
pub mod extensions;
pub mod fee;
pub mod proxy;
pub mod signature;
pub mod weights;

/// Macro to set a value (e.g. when using the `parameter_types` macro) to either a production value
/// or to an environment variable or testing value (in case the `frequency-local` feature is selected or in instant sealing mode).
/// Note that the environment variable is evaluated _at compile time_.
///
/// Usage:
/// ```Rust
/// parameter_types! {
///     // Note that the env variable version parameter cannot be const.
///     pub LaunchPeriod: BlockNumber = prod_or_testnet_or_local!(7 * DAYS, 28 * DAYS, 1 * MINUTES);
///     pub const VotingPeriod: BlockNumber = prod_or_testnet_or_local!(7 * DAYS, 28 * DAYS, 1 * MINUTES);
/// }
/// ```
#[macro_export]
macro_rules! prod_or_testnet_or_local {
	($prod:expr, $test:expr, $local:expr) => {
		if cfg!(any(feature = "frequency-local", feature = "frequency-no-relay",)) {
			$local
		} else if cfg!(feature = "frequency-testnet") {
			$test
		} else if cfg!(feature = "frequency-westend") {
			$test
		} else {
			$prod
		}
	};
}
