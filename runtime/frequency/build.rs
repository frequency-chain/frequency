#[cfg(all(feature = "std", feature = "metadata-hash"))]
fn main() {
	substrate_wasm_builder::WasmBuilder::init_with_defaults()
		.enable_metadata_hash("FRQCY", 8)
		.build();

	substrate_wasm_builder::WasmBuilder::init_with_defaults()
		.set_file_name("fast_runtime_binary.rs")
		.enable_feature("fast-runtime")
		.enable_metadata_hash("FRQCY", 8)
		.build();
}

#[cfg(all(feature = "std", not(feature = "metadata-hash")))]
fn main() {
	substrate_wasm_builder::WasmBuilder::new()
		.with_current_project()
		.export_heap_base()
		.import_memory()
		.build();
}

/// The wasm builder is deactivated when compiling
/// this crate for wasm to speed up the compilation.
#[cfg(not(feature = "std"))]
fn main() {}
