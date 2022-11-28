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

#[macro_export]
macro_rules! create_runtime_version_for_network {
	($runtime_apis:expr) => {
		use sp_version::RuntimeVersion;
		#[sp_version::runtime_version]
		const VERSION: RuntimeVersion = RuntimeVersion {
			spec_name: create_runtime_str_for_network!("frequency"),
			impl_name: create_runtime_str_for_network!("frequency"),
			authoring_version: 1,
			spec_version: 1,
			impl_version: 1,
			transaction_version: 1,
			state_version: 1,
			apis: runtime_apis,
		};
		VERSION
	};
}
