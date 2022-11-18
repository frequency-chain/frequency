mod rpc_mock;

use super::*;
use rpc_mock::*;

sp_api::mock_impl_runtime_apis! {
	impl MsaRuntimeApi<Block> for TestRuntimeApi {
		fn
	}
}
