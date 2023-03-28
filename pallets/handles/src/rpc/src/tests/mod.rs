mod rpc_mock;

use super::*;
use rpc_mock::*;

use pallet_handles_runtime_api::HandlesRuntimeApi;
use std::sync::Arc;
use substrate_test_runtime_client::runtime::Block;

const SUCCESSFUL_SCHEMA_ID: u16 = 1;

sp_api::mock_impl_runtime_apis! {
	impl HandlesRuntimeApi<Block> for TestRuntimeApi {
		fn get_handle_for_msa(&self, msa_id: MessageSourceId) -> Option<HandleResponse> {
			if msa_id == SUCCESSFUL_SCHEMA_ID {
				Some(HandleResponse {
					base: vec![],
					canonical: vec![],
					suffix: 0,
				})
			} else {
				None
			}
		}
	}
}
