mod rpc_mock;

use super::*;
use rpc_mock::*;

use common_primitives::node::BlockNumber;
use pallet_msa_runtime_api::MsaRuntimeApi;
use serde_json::json;
use std::sync::Arc;
use substrate_test_runtime_client::runtime::{AccountId, Block};
use DelegatorId;
use ProviderId;
use SchemaId;

// a hacky way to change up the results of the api calls
const NOBODY: MessageSourceId = 99;

sp_api::mock_impl_runtime_apis! {
	impl MsaRuntimeApi<Block, AccountId> for TestRuntimeApi {
		fn has_delegation(delegator: DelegatorId, _provider: ProviderId, _block_number: BlockNumber, _schema_id: Option<SchemaId>) -> bool {
			if (delegator == DelegatorId(NOBODY)) { return false; }
			true
		}

		fn get_granted_schemas_by_msa_id(delegator: DelegatorId, _provider: ProviderId) -> Option<Vec<SchemaId>> {
			if (delegator == DelegatorId(NOBODY)) { return None; }
			let result: Vec<SchemaId> = vec![1,2];
			Some(result)
		}

		fn get_public_key_count_by_msa_id(msa_id: MessageSourceId) -> u8 {
			if msa_id == NOBODY { return 0; }
			1
		}

		fn get_providers_for_msa_id(msa_id: MessageSourceId) -> Vec<ProviderId> {
			if msa_id == NOBODY { return vec![]; }
			vec![ProviderId(2),ProviderId(3),ProviderId(4)]
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

	let actual_delegations = json["capabilityDelegation"].as_array().unwrap();
	assert_eq!(3, actual_delegations.len());

	let expected = json!({
		"blockchainAccountId": 0,
		"controller": "did:dsnp:4",
		"id": "did:dsnp:4",
		"type": "sr25519"
	});
	assert_eq!(expected, actual_delegations[2]);
}
