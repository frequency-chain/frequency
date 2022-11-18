/// Runtime api/commands for the frequency node.
use sp_version::RuntimeVersion;

pub trait FrequencyRuntimeApi {
	fn get_runtime_version() -> RuntimeVersion;
}
