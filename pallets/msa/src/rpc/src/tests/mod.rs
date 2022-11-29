mod rpc_mock;

use super::*;
use rpc_mock::*;

use common_primitives::node::BlockNumber;
use pallet_msa_runtime_api::MsaRuntimeApi;
use serde_json::json;
use std::sync::Arc;
use substrate_test_runtime_client::runtime::{AccountId, Block};

sp_api::mock_impl_runtime_apis! {
	impl MsaRuntimeApi<Block, AccountId> for TestRuntimeApi {
		fn has_delegation(delegator: DelegatorId, provider: ProviderId, block_number: BlockNumber, schema_id: Option<SchemaId>) -> bool {
			false
		}


		fn get_granted_schemas_by_msa_id(delegator: DelegatorId, provider: ProviderId) -> Option<Vec<SchemaId>> {
			Some(vec![])
		}

		fn get_public_key_count_by_msa_id(msa_id: MessageSourceId) -> u8 {
			1
		}
	}
}

#[tokio::test]
async fn did_to_msa_id_works() {
	let client = Arc::new(TestApi {});
	let api = MsaHandler::new(client);

	let did = Vec::from("did:dsnp:1");

	let result = api.did_to_msa_id(did);
	assert_eq!(true, result.is_ok());
	assert_eq!(Some(1), result.unwrap());
}

#[tokio::test]
async fn msa_id_to_did_document_works() {
	let client = Arc::new(TestApi {});
	let api = MsaHandler::new(client);

	let result = api.msa_id_to_did_document(1u64);
	assert_eq!(true, result.is_ok());

	let unwrapped: String = result.unwrap().unwrap();

	let json: serde_json::Value = serde_json::from_str(&unwrapped.as_str()).unwrap();
	assert_eq!("did:dsnp:1", json["id"]);
	assert_eq!("did:dsnp:1", json["controller"]);
}
