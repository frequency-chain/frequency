// Duplicated here to reduce the build dependencies
#[allow(unused)]
const FREQUENCY_TESTNET_TOKEN: &str = "XRQCY";
#[allow(unused)]
const FREQUENCY_LOCAL_TOKEN: &str = "UNIT";
#[allow(unused)]
const FREQUENCY_TOKEN: &str = "FRQCY";
#[allow(unused)]
const TOKEN_DECIMALS: u8 = 8;

#[cfg(all(feature = "std", not(feature = "metadata-hash")))]
fn main() {
	substrate_wasm_builder::WasmBuilder::build_using_defaults()
}

#[cfg(all(feature = "std", feature = "metadata-hash", feature = "frequency"))]
fn main() {
	substrate_wasm_builder::WasmBuilder::init_with_defaults()
		.enable_metadata_hash(FREQUENCY_TOKEN, TOKEN_DECIMALS)
		.build()
}

#[cfg(all(feature = "std", feature = "metadata-hash", feature = "frequency-testnet"))]
fn main() {
	substrate_wasm_builder::WasmBuilder::init_with_defaults()
		.enable_metadata_hash(FREQUENCY_TESTNET_TOKEN, TOKEN_DECIMALS)
		.build()
}

#[cfg(all(
	feature = "std",
	feature = "metadata-hash",
	any(feature = "frequency-no-relay", feature = "frequency-local")
))]
fn main() {
	substrate_wasm_builder::WasmBuilder::init_with_defaults()
		.enable_metadata_hash(FREQUENCY_LOCAL_TOKEN, TOKEN_DECIMALS)
		.build()
}

/// The wasm builder is deactivated when compiling
/// this crate for wasm to speed up the compilation.
#[cfg(not(feature = "std"))]
fn main() {}
