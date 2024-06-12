mod rpc_mock;

use super::*;
use rpc_mock::*;

use common_primitives::node::{AccountId, Block, BlockNumber};
use pallet_msa_runtime_api::MsaRuntimeApi;
use parity_scale_codec::Encode;
use sp_api::offchain::testing::TestPersistentOffchainDB;
use sp_core::offchain::OffchainStorage;
use sp_runtime::traits::Zero;
use std::{sync::Arc, vec};

const PROVIDER_WITH_DELEGATE_A: ProviderId = ProviderId(1);
const DELEGATE_A: DelegatorId = DelegatorId(2);
const DELEGATE_B: DelegatorId = DelegatorId(3);
const PROVIDER_WITH_DELEGATE_A_AND_B: ProviderId = ProviderId(4);
const SCHEMA_FOR_A: u16 = 1;
const SCHEMA_FOR_A_AND_B: u16 = 2;
const SCHEMA_FOR_B: u16 = 3;
const NOT_EXIST_MSA: u64 = 100;

sp_api::mock_impl_runtime_apis! {
	impl MsaRuntimeApi<Block, AccountId> for TestRuntimeApi {
		fn has_delegation(delegator: DelegatorId, provider: ProviderId, block_number: BlockNumber, schema_id: Option<SchemaId>) -> bool {
			if block_number > 1000 {
				return false
			}
			match (delegator, provider, schema_id) {
				(DELEGATE_A, PROVIDER_WITH_DELEGATE_A, Some(SCHEMA_FOR_A)) => true,
				(DELEGATE_A, PROVIDER_WITH_DELEGATE_A_AND_B, Some(SCHEMA_FOR_A_AND_B)) => true,
				(DELEGATE_B, PROVIDER_WITH_DELEGATE_A_AND_B, Some(SCHEMA_FOR_A_AND_B)) => true,
				(DELEGATE_B, PROVIDER_WITH_DELEGATE_A_AND_B, Some(SCHEMA_FOR_B)) => true,
				_ => false,
			}
		}

		/// Get the list of schema ids (if any) that exist in any delegation between the delegator and provider
		fn get_granted_schemas_by_msa_id(delegator: DelegatorId, provider: ProviderId) -> Option<Vec<SchemaGrant<SchemaId, BlockNumber>>>{
			match (delegator, provider) {
				(DELEGATE_A, PROVIDER_WITH_DELEGATE_A) => Some(vec![SchemaGrant::new(SCHEMA_FOR_A, BlockNumber::zero())]),
				(DELEGATE_A, PROVIDER_WITH_DELEGATE_A_AND_B) => Some(vec![SchemaGrant::new(SCHEMA_FOR_A_AND_B, BlockNumber::zero())]),
				(DELEGATE_B, PROVIDER_WITH_DELEGATE_A_AND_B) => Some(vec![SchemaGrant::new(SCHEMA_FOR_A_AND_B, BlockNumber::zero()), SchemaGrant::new(SCHEMA_FOR_B, BlockNumber::zero())]),
				_ => None,
			}
		}

		/// Get the list of all delegations and grants
		fn get_all_granted_delegations_by_msa_id(delegator: DelegatorId) -> Vec<DelegationResponse<SchemaId, BlockNumber>> {
			match delegator {
				DELEGATE_A => vec![DelegationResponse{ provider_id: ProviderId(1), permissions: vec![SchemaGrant::new(SCHEMA_FOR_A, BlockNumber::zero())]}],
				DELEGATE_B => vec![
					DelegationResponse{ provider_id: ProviderId(2), permissions: vec![SchemaGrant::new(SCHEMA_FOR_A_AND_B, BlockNumber::zero()), SchemaGrant::new(SCHEMA_FOR_B, BlockNumber::zero())]},
					DelegationResponse{ provider_id: ProviderId(3), permissions: vec![SchemaGrant::new(SCHEMA_FOR_A_AND_B, BlockNumber::zero())]}
					],
				_ => vec![],
			}
		}
	}
}

#[tokio::test]
async fn check_delegations_can_success_with_multiple() {
	let client = Arc::new(TestApi {});
	let api = MsaHandler::<TestApi, Block, TestPersistentOffchainDB>::new(client, None);

	let result = api.check_delegations(
		vec![DELEGATE_A, DELEGATE_B],
		PROVIDER_WITH_DELEGATE_A_AND_B,
		100,
		Some(SCHEMA_FOR_A_AND_B),
	);

	assert_eq!(true, result.is_ok());
	let arr = result.unwrap();
	assert_eq!(vec![(DELEGATE_A, true), (DELEGATE_B, true)], arr);
}

#[tokio::test]
async fn check_delegations_with_good_and_bad_responses() {
	let client = Arc::new(TestApi {});
	let api = MsaHandler::<TestApi, Block, TestPersistentOffchainDB>::new(client, None);

	let result = api.check_delegations(
		vec![DELEGATE_A, DELEGATE_B],
		PROVIDER_WITH_DELEGATE_A,
		100,
		Some(SCHEMA_FOR_A),
	);

	assert_eq!(true, result.is_ok());
	let arr = result.unwrap();
	assert_eq!(vec![(DELEGATE_A, true), (DELEGATE_B, false)], arr);
}

#[tokio::test]
async fn check_delegations_with_bad_delegate_msa() {
	let client = Arc::new(TestApi {});
	let api = MsaHandler::<TestApi, Block, TestPersistentOffchainDB>::new(client, None);

	let result = api.check_delegations(
		vec![DelegatorId(NOT_EXIST_MSA)],
		PROVIDER_WITH_DELEGATE_A,
		100,
		Some(SCHEMA_FOR_A_AND_B),
	);

	assert_eq!(true, result.is_ok());
	let arr = result.unwrap();
	assert_eq!(vec![(DelegatorId(NOT_EXIST_MSA), false)], arr);
}

#[tokio::test]
async fn check_delegations_with_bad_provider() {
	let client = Arc::new(TestApi {});
	let api = MsaHandler::<TestApi, Block, TestPersistentOffchainDB>::new(client, None);

	let result = api.check_delegations(
		vec![DELEGATE_A, DELEGATE_B],
		ProviderId(NOT_EXIST_MSA),
		100,
		Some(SCHEMA_FOR_A_AND_B),
	);

	assert_eq!(true, result.is_ok());
	let arr = result.unwrap();
	assert_eq!(vec![(DELEGATE_A, false), (DELEGATE_B, false)], arr);
}

#[tokio::test]
async fn check_delegations_returns_fail_if_after_block() {
	let client = Arc::new(TestApi {});
	let api = MsaHandler::<TestApi, Block, TestPersistentOffchainDB>::new(client, None);

	let result = api.check_delegations(
		vec![DELEGATE_A, DELEGATE_B],
		PROVIDER_WITH_DELEGATE_A_AND_B,
		1001,
		Some(SCHEMA_FOR_A_AND_B),
	);

	assert_eq!(true, result.is_ok());
	let arr = result.unwrap();
	assert_eq!(vec![(DELEGATE_A, false), (DELEGATE_B, false)], arr);
}

#[tokio::test]
async fn get_granted_schemas_by_msa_id_with_success() {
	let client = Arc::new(TestApi {});
	let api = MsaHandler::<TestApi, Block, TestPersistentOffchainDB>::new(client, None);

	let result = api.get_granted_schemas_by_msa_id(DELEGATE_A, PROVIDER_WITH_DELEGATE_A);

	assert_eq!(true, result.is_ok());
	let response = result.unwrap().unwrap();
	assert_eq!(vec![SchemaGrant::new(SCHEMA_FOR_A, BlockNumber::zero())], response);
}

#[tokio::test]
async fn get_all_granted_delegations_by_msa_id_with_success() {
	let client = Arc::new(TestApi {});
	let api = MsaHandler::<TestApi, Block, TestPersistentOffchainDB>::new(client, None);

	let result = api.get_all_granted_delegations_by_msa_id(DELEGATE_B);

	assert_eq!(true, result.is_ok());
	let response = result.unwrap();
	assert_eq!(
		vec![
			DelegationResponse {
				provider_id: ProviderId(2),
				permissions: vec![
					SchemaGrant::new(SCHEMA_FOR_A_AND_B, BlockNumber::zero()),
					SchemaGrant::new(SCHEMA_FOR_B, BlockNumber::zero())
				]
			},
			DelegationResponse {
				provider_id: ProviderId(3),
				permissions: vec![SchemaGrant::new(SCHEMA_FOR_A_AND_B, BlockNumber::zero())]
			}
		],
		response
	);
}

#[tokio::test]
async fn get_granted_schemas_by_msa_id_with_none() {
	let client = Arc::new(TestApi {});
	let api = MsaHandler::<TestApi, Block, TestPersistentOffchainDB>::new(client, None);

	let result = api.get_granted_schemas_by_msa_id(DELEGATE_B, PROVIDER_WITH_DELEGATE_A_AND_B);

	assert_eq!(true, result.is_ok());
	let response = result.unwrap().unwrap();
	assert_eq!(
		vec![
			SchemaGrant::new(SCHEMA_FOR_A_AND_B, BlockNumber::zero()),
			SchemaGrant::new(SCHEMA_FOR_B, BlockNumber::zero())
		],
		response
	);
}

#[tokio::test]
async fn get_granted_schemas_by_msa_id_with_no_delegation() {
	let client = Arc::new(TestApi {});
	let api = MsaHandler::<TestApi, Block, TestPersistentOffchainDB>::new(client, None);

	let result = api.get_granted_schemas_by_msa_id(DELEGATE_B, PROVIDER_WITH_DELEGATE_A);

	assert_eq!(true, result.is_ok());
	let response = result.unwrap();
	assert!(response.is_none());
}

#[tokio::test]
async fn get_granted_schemas_by_msa_id_with_bad_provider_id() {
	let client = Arc::new(TestApi {});
	let api = MsaHandler::<TestApi, Block, TestPersistentOffchainDB>::new(client, None);

	let result = api.get_granted_schemas_by_msa_id(DELEGATE_A, ProviderId(NOT_EXIST_MSA));

	assert_eq!(true, result.is_ok());
	let response = result.unwrap();
	assert!(response.is_none());
}

#[tokio::test]
async fn get_keys_by_msa_id_with_disabled_offchain_should_fail() {
	let client = Arc::new(TestApi {});
	let api = MsaHandler::<TestApi, Block, TestPersistentOffchainDB>::new(client, None);

	let result = api.get_keys_by_msa_id(NOT_EXIST_MSA);

	assert_eq!(true, result.is_err());
	assert_eq!("Custom error: OffchainIndexingNotEnabled", result.unwrap_err().to_string());
}

#[tokio::test]
async fn get_keys_by_msa_id_with_empty_value_should_work() {
	let client = Arc::new(TestApi {});
	let api = MsaHandler::<TestApi, Block, TestPersistentOffchainDB>::new(
		client,
		Some(TestPersistentOffchainDB::new()),
	);

	let result = api.get_keys_by_msa_id(NOT_EXIST_MSA);

	assert_eq!(true, result.is_ok());
	let response = result.unwrap();
	assert_eq!(None, response);
}

#[tokio::test]
async fn get_keys_by_msa_id_with_value_should_work() {
	let msa_id: MessageSourceId = 10;
	let accounts =
		vec![AccountId::from([1u8; 32]), AccountId::from([2u8; 32]), AccountId::from([5u8; 32])];
	let client = Arc::new(TestApi {});
	let mut db = TestPersistentOffchainDB::new();
	db.set(
		sp_offchain::STORAGE_PREFIX,
		&get_msa_account_storage_key_name(msa_id),
		&accounts.encode(),
	);
	let api = MsaHandler::<TestApi, Block, TestPersistentOffchainDB>::new(client, Some(db));

	let result = api.get_keys_by_msa_id(msa_id);

	assert_eq!(true, result.is_ok());
	let response = result.unwrap();
	assert_eq!(true, response.is_some());
	assert_eq!(KeyInfoResponse { msa_id, msa_keys: accounts }, response.unwrap());
}
