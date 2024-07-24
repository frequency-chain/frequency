#[cfg(all(feature = "std", not(feature = "metadata-hash")))]
fn main() {
	substrate_wasm_builder::WasmBuilder::build_using_defaults()
}

#[cfg(all(feature = "std", feature = "metadata-hash"))]
fn main() {
	substrate_wasm_builder::WasmBuilder::init_with_defaults()
		.enable_metadata_hash("FRQCY", 8)
		.build()
}

#[cfg(not(feature = "std"))]
fn main() {}
