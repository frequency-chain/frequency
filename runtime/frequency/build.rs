use substrate_wasm_builder::WasmBuilder;

fn main() {
	// VSCode Users: Uncomment the following line to disable the ANSI color codes.
	// The OUTPUT pane does not understand ANSI color codes and will show garbage without this.
	// std::env::set_var("WASM_BUILD_NO_COLOR", "1");
	WasmBuilder::new()
		.with_current_project()
		.export_heap_base()
		.import_memory()
		.build()
}
