use frame_support::{assert_ok, pallet_prelude::Hooks, traits::OriginTrait};

use crate::{
	get_bucket_number, tests::mock::*, Config, DispatchResult, FinalizedBlockResponse,
	IndexedEvent, Pallet, LAST_PROCESSED_BLOCK_STORAGE_NAME, MSA_INITIAL_INDEXED_STORAGE_NAME,
	RPC_FINALIZED_BLOCK_REQUEST_BODY, RPC_FINALIZED_BLOCK_REQUEST_URL,
};
use frame_system::pallet_prelude::BlockNumberFor;
use parity_scale_codec::{Decode, Encode};

use common_primitives::{
	msa::MessageSourceId,
	node::AccountId,
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

fn retire_accounts<T: Config>(accounts: Vec<T::AccountId>)
where
	<T as frame_system::Config>::AccountId: From<sr25519::Public>,
{
	for account in accounts {
		assert_ok!(Pallet::<T>::retire_msa(T::RuntimeOrigin::signed(account.into())));
	}
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
		for (i, account) in accounts.iter().enumerate() {
			let msa_key = get_msa_account_storage_key_name((i + 1) as MessageSourceId);
			let result = get_index_value::<Vec<AccountId32>>(&msa_key);
			assert_eq!(result, Ok(Some(vec![account.clone()])));
		}

		let last_processed_block =
			get_index_value::<BlockNumberFor<Test>>(&LAST_PROCESSED_BLOCK_STORAGE_NAME[..]);
		assert_eq!(last_processed_block, Ok(Some(block_number)));
	});

	ext.execute_with(|| {
		block_number += 1;
		System::set_block_number(block_number);
		retire_accounts::<Test>(accounts.clone());
		let response = FinalizedBlockResponse {
			result: "0x5685c63b9df72b59f6fa8e1223532c041d15a1abbe39a6d2a48d6565a091839d"
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
	});

	ext.persist_offchain_overlay();

	ext.execute_with(|| {
		// act
		Msa::offchain_worker(block_number);

		// assert
		for (i, _) in accounts.into_iter().enumerate() {
			let msa_key = get_msa_account_storage_key_name((i + 1) as MessageSourceId);
			let result = get_index_value::<Vec<AccountId32>>(&msa_key);
			assert_eq!(result, Ok(None));
		}

		let last_processed_block =
			get_index_value::<BlockNumberFor<Test>>(&LAST_PROCESSED_BLOCK_STORAGE_NAME[..]);
		assert_eq!(last_processed_block, Ok(Some(block_number)));
	});
}

#[test]
fn get_bucket_number_should_return_sudo_random_value() {
	new_test_ext().execute_with(|| {
		let event1: IndexedEvent<Test> = IndexedEvent::IndexedMsaCreated {
			msa_id: 100,
			key: AccountId::new([
				1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 1u8, 2, 3, 4, 5,
				6, 7, 8, 9, 10, 1, 2,
			]),
		};
		let event2: IndexedEvent<Test> = IndexedEvent::IndexedMsaCreated {
			msa_id: 101,
			key: AccountId::new([
				1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 1u8, 2, 3, 4, 5,
				6, 7, 8, 9, 10, 1, 2,
			]),
		};

		let bucket_1 = get_bucket_number(&event1);
		let bucket_2 = get_bucket_number(&event2);

		assert_ne!(bucket_1, bucket_2);
		assert!(1 <= bucket_1 && bucket_1 <= 1000);
		assert!(1 <= bucket_2 && bucket_2 <= 1000);
	});
}

#[test]
fn reindex_offchain_should_always_succeed() {
	new_test_ext().execute_with(|| {
		let (new_msa_id, key_pair) = create_account();

		assert_ok!(Msa::reindex_offchain(
			test_origin_signed(1),
			new_msa_id,
			Some(key_pair.public().into()),
		));

		assert_ok!(Msa::reindex_offchain(test_origin_signed(1), 10u64, None,));
	})
}
