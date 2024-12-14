// Succinct Proof of Concept
// #![no_main]
// use sp1_zkvm;
// #![feature(core_intrinsics)]
use core::time::Duration;

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
	// Magic to attach debugger to the build process
	// from here: https://github.com/vadimcn/codelldb/blob/master/MANUAL.md#attaching-debugger-to-the-current-process-rust
	let url = format!("vscode://vadimcn.vscode-lldb/launch/config?{{'request':'attach','pid':{}}}", std::process::id());
	std::process::Command::new("code").arg("--open-url").arg(url).output().unwrap();
	let one_sec = Duration::from_millis(10000);
	std::thread::sleep(one_sec); // Wait for debugger to attach
	substrate_wasm_builder::WasmBuilder::build_using_defaults()
}

#[cfg(all(feature = "std", feature = "metadata-hash", feature = "frequency"))]
fn main() {
	// Magic to attach debugger to the build process
	// from here: https://github.com/vadimcn/codelldb/blob/master/MANUAL.md#attaching-debugger-to-the-current-process-rust
	let url = format!("vscode://vadimcn.vscode-lldb/launch/config?{{'request':'attach','pid':{}}}", std::process::id());
	std::process::Command::new("code").arg("--open-url").arg(url).output().unwrap();
	let one_sec = Duration::from_millis(10000);
	std::thread::sleep(one_sec); // Wait for debugger to attach
	substrate_wasm_builder::WasmBuilder::init_with_defaults()
		.enable_metadata_hash(FREQUENCY_TOKEN, TOKEN_DECIMALS)
		.build()
}

#[cfg(all(feature = "std", feature = "metadata-hash", feature = "frequency-testnet"))]
fn main() {
	// Magic to attach debugger to the build process
	// from here: https://github.com/vadimcn/codelldb/blob/master/MANUAL.md#attaching-debugger-to-the-current-process-rust
	let url = format!("vscode://vadimcn.vscode-lldb/launch/config?{{'request':'attach','pid':{}}}", std::process::id());
	std::process::Command::new("code").arg("--open-url").arg(url).output().unwrap();
	let one_sec = Duration::from_millis(10000);
	std::thread::sleep(one_sec); // Wait for debugger to attach
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
	// Magic to attach debugger to the build process
	// from here: https://github.com/vadimcn/codelldb/blob/master/MANUAL.md#attaching-debugger-to-the-current-process-rust
	let url = format!("vscode://vadimcn.vscode-lldb/launch/config?{{'request':'attach','pid':{}}}", std::process::id());
	std::process::Command::new("code").arg("--open-url").arg(url).output().unwrap();
	let one_sec = Duration::from_millis(10000);
	std::thread::sleep(one_sec); // Wait for debugger to attach
	// std::intrinsics::breakpoint();
	substrate_wasm_builder::WasmBuilder::init_with_defaults()
		.enable_metadata_hash(FREQUENCY_LOCAL_TOKEN, TOKEN_DECIMALS)
		.build()
}

/// The wasm builder is deactivated when compiling
/// this crate for wasm to speed up the compilation.
#[cfg(not(feature = "std"))]
fn main() {}

// sp1_zkvm::entrypoint!(main);
