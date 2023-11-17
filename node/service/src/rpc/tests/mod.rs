mod rpc_mock;

use rpc_mock::*;

use crate::rpc::frequency_rpc::{FrequencyRpcApiServer, FrequencyRpcHandler};
use common_primitives::rpc::RpcEvent;
use frame_system::{EventRecord, Phase};
use parity_scale_codec::{Decode, Encode};
use sp_core::H256;
use sp_runtime::scale_info::TypeInfo;
use std::sync::Arc;
use substrate_test_runtime_client::runtime::Block;
use system_runtime_api::AdditionalRuntimeApi;

#[derive(Debug, TypeInfo, Clone, Encode, Decode, Eq, PartialEq)]
enum FakeRuntime {
	#[codec(index = 60)]
	FakePallet(FakePalletEvents),
}

#[derive(Debug, TypeInfo, Clone, Encode, Decode, Eq, PartialEq)]
enum FakePalletEvents {
	#[codec(index = 7)]
	FakeEvent { msa_id: u64 },
}

sp_api::mock_impl_runtime_apis! {
	impl AdditionalRuntimeApi<Block> for TestRuntimeApi {
		fn get_events() -> Vec<RpcEvent> {
			let event: EventRecord<FakeRuntime, ()> = EventRecord {
				phase: Phase::ApplyExtrinsic(5),
				event: FakeRuntime::FakePallet(FakePalletEvents::FakeEvent { msa_id: 42 }),
				topics: vec![],
			};
			vec![event.into()]
		}
	}
}

type GetEventsResult = Result<Vec<RpcEvent>, jsonrpsee::core::Error>;

#[tokio::test]
async fn get_events_should_work() {
	let client = Arc::new(TestApi {});
	let api = FrequencyRpcHandler::new(client);

	let result: GetEventsResult = api.get_events(H256::random());

	assert_eq!(false, result.is_err());

	assert_eq!(
		result.unwrap()[0],
		RpcEvent { phase: Some(5), pallet: 60, event: 7, data: vec![42, 0, 0, 0, 0, 0, 0, 0] }
	);
}
