use frame_support::{assert_ok, pallet_prelude::Hooks, traits::OriginTrait};

use frame_system::pallet_prelude::BlockNumberFor;
use parity_scale_codec::{Decode, Encode};

use crate::{
	tests::mock::*, Config, DispatchResult, FinalizedBlockResponse, Pallet,
	LAST_PROCESSED_BLOCK_STORAGE_NAME, MSA_INITIAL_INDEXED_STORAGE_NAME,
	RPC_FINALIZED_BLOCK_REQUEST_BODY, RPC_FINALIZED_BLOCK_REQUEST_URL,
};

use common_primitives::{
	msa::MessageSourceId,
	offchain::{get_index_value, get_msa_account_storage_key_name},
};
use pretty_assertions::assert_eq;
use sp_core::{crypto::AccountId32, offchain::testing::PendingRequest, sr25519, Pair};
use sp_runtime::offchain::storage::StorageValueRef;

fn fill_accounts<T: Config>(accounts: usize, with_indexing: bool) -> Vec<T::AccountId>
where
	<T as frame_system::Config>::AccountId: From<sr25519::Public>,
{
	let mut result = vec![];
	for _ in 1..=accounts {
		let (key_pair, _) = sr25519::Pair::generate();
		let account = key_pair.public();
		let public_key = T::AccountId::decode(&mut &account.encode()[..]).expect("should decode");
		result.push(public_key.clone());
		match with_indexing {
			false => {
				assert_ok!(Pallet::<T>::create_account(public_key, |_| -> DispatchResult {
					Ok(())
				}));
			},
			true => {
				assert_ok!(Pallet::<T>::create(T::RuntimeOrigin::signed(account.into())));
			},
		}
	}
	result
}

#[test]
pub fn offchain_worker_should_populate_accounts_with_initial_state() {
	let (mut ext, state) = new_test_with_offchain_ext();
	ext.execute_with(|| {
		// arrange
		let accounts = fill_accounts::<Test>(10, false);
		let block_number = BlockNumberFor::<Test>::from(10u32);
		let response = FinalizedBlockResponse {
			result: "0x5685c63b9df72b59f6fa8e1223532c041d15a1abbe39a6d2a48d6565a091839b"
				.to_string(),
		};
		let decoded_from_hex = hex::decode(&response.result[2..]).expect("should decode hex");
		let val = <<Test as frame_system::Config>::Hash>::decode(&mut &decoded_from_hex[..])
			.expect("should decode hash");
		frame_system::BlockHash::<Test>::set(block_number, val);
		let serialized_block = serde_json::to_string(&response).expect("should serialize");
		let response_bytes = serialized_block.as_bytes().to_vec();
		state.write().expect_request(PendingRequest {
			method: "POST".into(),
			uri: RPC_FINALIZED_BLOCK_REQUEST_URL.into(),
			headers: vec![("Content-Type".into(), "application/json".into())],
			sent: true,
			body: RPC_FINALIZED_BLOCK_REQUEST_BODY.to_vec(),
			response: Some(response_bytes),
			..Default::default()
		});

		// act
		Msa::offchain_worker(block_number);

		// assert
		for (i, account) in accounts.into_iter().enumerate() {
			let msa_key = get_msa_account_storage_key_name((i + 1) as MessageSourceId);
			let result = get_index_value::<Vec<AccountId32>>(&msa_key);
			assert_eq!(result, Ok(Some(vec![account])));
		}

		let last_processed_block =
			get_index_value::<BlockNumberFor<Test>>(&LAST_PROCESSED_BLOCK_STORAGE_NAME[..]);
		assert_eq!(last_processed_block, Ok(Some(block_number)));

		let initial_state_indexed = get_index_value::<bool>(&MSA_INITIAL_INDEXED_STORAGE_NAME[..]);
		assert_eq!(initial_state_indexed, Ok(Some(true)));
	});
}

#[test]
pub fn offchain_worker_should_populate_accounts_with_offchain_indexed_events() {
	// arrange
	let (mut ext, state) = new_test_with_offchain_ext();
	let mut accounts = vec![];
	let mut block_number = 0;
	ext.execute_with(|| {
		accounts = fill_accounts::<Test>(10, true);
		block_number = BlockNumberFor::<Test>::from(1u32);
		let response = FinalizedBlockResponse {
			result: "0x5685c63b9df72b59f6fa8e1223532c041d15a1abbe39a6d2a48d6565a091839b"
				.to_string(),
		};
		let decoded_from_hex = hex::decode(&response.result[2..]).expect("should decode hex");
		let val = <<Test as frame_system::Config>::Hash>::decode(&mut &decoded_from_hex[..])
			.expect("should decode hash");
		frame_system::BlockHash::<Test>::set(block_number, val);
		let serialized_block = serde_json::to_string(&response).expect("should serialize");
		let response_bytes = serialized_block.as_bytes().to_vec();
		state.write().expect_request(PendingRequest {
			method: "POST".into(),
			uri: RPC_FINALIZED_BLOCK_REQUEST_URL.into(),
			headers: vec![("Content-Type".into(), "application/json".into())],
			sent: true,
			body: RPC_FINALIZED_BLOCK_REQUEST_BODY.to_vec(),
			response: Some(response_bytes),
			..Default::default()
		});
		let storage = StorageValueRef::persistent(MSA_INITIAL_INDEXED_STORAGE_NAME);
		storage.set(&true);
	});

	ext.persist_offchain_overlay();

	ext.execute_with(|| {
		// act
		Msa::offchain_worker(block_number);

		// assert
		for (i, account) in accounts.into_iter().enumerate() {
			let msa_key = get_msa_account_storage_key_name((i + 1) as MessageSourceId);
			let result = get_index_value::<Vec<AccountId32>>(&msa_key);
			assert_eq!(result, Ok(Some(vec![account])));
		}

		let last_processed_block =
			get_index_value::<BlockNumberFor<Test>>(&LAST_PROCESSED_BLOCK_STORAGE_NAME[..]);
		assert_eq!(last_processed_block, Ok(Some(block_number)));
	});
}
