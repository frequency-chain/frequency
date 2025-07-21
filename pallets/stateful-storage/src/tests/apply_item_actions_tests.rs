use crate::{
	stateful_child_tree::StatefulChildTree,
	test_common::{constants::*, test_utility::*},
	tests::mock::*,
	types::*,
	Config, Error, Event as StatefulEvent,
};
use common_primitives::{
	stateful_storage::{PageHash, PageNonce},
	utils::wrap_binary_data,
};
use frame_support::{assert_err, assert_ok, BoundedVec};
use parity_scale_codec::Encode;
#[allow(unused_imports)]
use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};
use sp_core::{sr25519, Get, Pair};
use sp_runtime::{AccountId32, MultiSignature};

#[test]
fn apply_item_actions_with_invalid_msa_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let msa_id = 1;
		let caller_1 = test_public(INVALID_MSA_ID); // hard-coded in mocks to return None for MSA
		let schema_id = ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let actions = vec![ItemAction::Add { data: payload.try_into().unwrap() }];

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				NONEXISTENT_PAGE_HASH,
				BoundedVec::try_from(actions).unwrap(),
			),
			Error::<Test>::InvalidMessageSourceAccount
		)
	});
}

#[test]
fn apply_item_actions_with_invalid_schema_id_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = INVALID_SCHEMA_ID;
		let payload = vec![1; 5];
		let actions = vec![ItemAction::Add { data: payload.try_into().unwrap() }];

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				NONEXISTENT_PAGE_HASH,
				BoundedVec::try_from(actions).unwrap(),
			),
			Error::<Test>::InvalidSchemaId
		)
	});
}

#[test]
fn apply_item_actions_with_invalid_schema_location_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = PAGINATED_SCHEMA;
		let payload = vec![1; 5];
		let actions = vec![ItemAction::Add { data: payload.try_into().unwrap() }];

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				NONEXISTENT_PAGE_HASH,
				BoundedVec::try_from(actions).unwrap(),
			),
			Error::<Test>::SchemaPayloadLocationMismatch
		)
	});
}

#[test]
fn apply_item_actions_with_no_delegation_and_different_caller_from_owner_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let msa_id = 1;
		let caller_1 = test_public(2);
		let schema_id = UNDELEGATED_ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let actions = vec![ItemAction::Add { data: payload.try_into().unwrap() }];

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				NONEXISTENT_PAGE_HASH,
				BoundedVec::try_from(actions).unwrap(),
			),
			Error::<Test>::UnauthorizedDelegate
		)
	});
}

#[test]
fn apply_item_actions_with_corrupted_state_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let page: ItemizedPage<Test> = generate_page(None, None);
		let actions1 = vec![ItemAction::Add { data: payload.clone().try_into().unwrap() }];
		let key = (schema_id,);
		StatefulChildTree::<<Test as Config>::KeyHasher>::write(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			ITEMIZED_STORAGE_PREFIX,
			&key,
			&page,
		);
		let previous_page: ItemizedPage<Test> =
			StatefulChildTree::<<Test as Config>::KeyHasher>::try_read(
				&msa_id,
				PALLET_STORAGE_PREFIX,
				ITEMIZED_STORAGE_PREFIX,
				&key,
			)
			.unwrap()
			.unwrap();
		let previous_hash = previous_page.get_hash();

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				previous_hash,
				BoundedVec::try_from(actions1).unwrap(),
			),
			Error::<Test>::CorruptedState
		)
	});
}

#[test]
fn apply_item_actions_initial_state_with_stale_hash_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let actions1 = vec![ItemAction::Add { data: payload.try_into().unwrap() }];

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				1u32, // any non-zero value
				BoundedVec::try_from(actions1).unwrap(),
			),
			Error::<Test>::StalePageState
		)
	});
}

#[test]
fn apply_item_actions_initial_state_with_valid_input_should_update_storage() {
	new_test_ext().execute_with(|| {
		// arrange
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let prev_content_hash: PageHash = 0;
		let actions = vec![ItemAction::Add { data: payload.try_into().unwrap() }];

		// act
		assert_ok!(StatefulStoragePallet::apply_item_actions(
			RuntimeOrigin::signed(caller_1),
			msa_id,
			schema_id,
			prev_content_hash,
			BoundedVec::try_from(actions).unwrap(),
		));

		// assert
		let updated_page: Option<ItemizedPage<Test>> =
			StatefulChildTree::<<Test as Config>::KeyHasher>::try_read(
				&msa_id,
				PALLET_STORAGE_PREFIX,
				ITEMIZED_STORAGE_PREFIX,
				&(schema_id,),
			)
			.unwrap();
		assert!(updated_page.is_some());
		let updated_page = updated_page.unwrap();
		let curr_content_hash = updated_page.get_hash();
		assert_eq!(updated_page.nonce, PageNonce::default() + 1);
		System::assert_last_event(
			StatefulEvent::ItemizedPageUpdated {
				msa_id,
				schema_id,
				prev_content_hash,
				curr_content_hash,
			}
			.into(),
		);
	});
}

#[test]
fn apply_item_actions_existing_page_with_valid_input_should_update_storage() {
	new_test_ext().execute_with(|| {
		// arrange
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let nonce = 10;
		let actions = vec![ItemAction::Add { data: payload.clone().try_into().unwrap() }];
		let page = create_itemized_page_from::<Test>(Some(nonce), &[payload.try_into().unwrap()]);
		let prev_content_hash = page.get_hash();
		let key = (schema_id,);

		// act
		<StatefulChildTree>::write(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			ITEMIZED_STORAGE_PREFIX,
			&key,
			&page,
		);
		assert_ok!(StatefulStoragePallet::apply_item_actions(
			RuntimeOrigin::signed(caller_1),
			msa_id,
			schema_id,
			prev_content_hash,
			BoundedVec::try_from(actions).unwrap(),
		));

		// assert
		let updated_page: Option<ItemizedPage<Test>> =
			StatefulChildTree::<<Test as Config>::KeyHasher>::try_read(
				&msa_id,
				PALLET_STORAGE_PREFIX,
				ITEMIZED_STORAGE_PREFIX,
				&(schema_id,),
			)
			.unwrap();
		assert!(updated_page.is_some());
		let updated_page = updated_page.unwrap();
		let curr_content_hash = updated_page.get_hash();
		assert_eq!(nonce + 1, updated_page.nonce);
		System::assert_last_event(
			StatefulEvent::ItemizedPageUpdated {
				msa_id,
				schema_id,
				prev_content_hash,
				curr_content_hash,
			}
			.into(),
		);
	});
}

#[test]
fn apply_item_actions_with_valid_input_and_empty_items_should_remove_storage() {
	new_test_ext().execute_with(|| {
		// arrange
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let actions1 = vec![ItemAction::Add { data: payload.try_into().unwrap() }];
		let actions2 = vec![ItemAction::Delete { index: 0 }];
		let keys = (schema_id,);
		assert_ok!(StatefulStoragePallet::apply_item_actions(
			RuntimeOrigin::signed(caller_1.clone()),
			msa_id,
			schema_id,
			NONEXISTENT_PAGE_HASH,
			BoundedVec::try_from(actions1).unwrap(),
		));

		let items1: Option<ItemizedPage<Test>> =
			StatefulChildTree::<<Test as Config>::KeyHasher>::try_read(
				&msa_id,
				PALLET_STORAGE_PREFIX,
				ITEMIZED_STORAGE_PREFIX,
				&keys,
			)
			.unwrap();
		assert!(items1.is_some());
		let content_hash = items1.unwrap().get_hash();

		// act
		assert_ok!(StatefulStoragePallet::apply_item_actions(
			RuntimeOrigin::signed(caller_1),
			msa_id,
			schema_id,
			content_hash,
			BoundedVec::try_from(actions2).unwrap(),
		));

		// assert
		let items2: Option<ItemizedPage<Test>> =
			StatefulChildTree::<<Test as Config>::KeyHasher>::try_read(
				&msa_id,
				PALLET_STORAGE_PREFIX,
				ITEMIZED_STORAGE_PREFIX,
				&(schema_id,),
			)
			.unwrap();
		assert!(items2.is_none());
		System::assert_last_event(
			StatefulEvent::ItemizedPageDeleted {
				msa_id,
				schema_id,
				prev_content_hash: content_hash,
			}
			.into(),
		);
	});
}

#[test]
fn apply_item_actions_on_signature_schema_fails_for_non_owner() {
	new_test_ext().execute_with(|| {
		// arrange
		// Note: normal use case for this test would be called by a delegate;
		// we don't bother setting up the delegation because the call should fail
		// before we check the delegation, as long as the owner_msa_id != caller_msa_id
		let (caller_msa_id, caller_keys) = get_signature_account();
		let owner_msa_id = caller_msa_id.saturating_add(1);
		let caller_1: AccountId32 = caller_keys.public().into();
		let schema_id = ITEMIZED_SIGNATURE_REQUIRED_SCHEMA;
		let payload = vec![1; 5];
		let actions1 = vec![ItemAction::Add { data: payload.try_into().unwrap() }];
		assert_err!(
			StatefulStoragePallet::apply_item_actions(
				RuntimeOrigin::signed(caller_1),
				owner_msa_id,
				schema_id,
				NONEXISTENT_PAGE_HASH,
				BoundedVec::try_from(actions1).unwrap(),
			),
			Error::<Test>::UnsupportedOperationForSchema
		);
	});
}

#[test]
fn apply_item_actions_on_signature_schema_succeeds_for_owner() {
	new_test_ext().execute_with(|| {
		// arrange
		let (msa_id, caller_keys) = get_signature_account();
		let caller_1: AccountId32 = caller_keys.public().into();
		let schema_id = ITEMIZED_SIGNATURE_REQUIRED_SCHEMA;
		let payload = vec![1; 5];
		let actions1 = vec![ItemAction::Add { data: payload.try_into().unwrap() }];
		assert_ok!(StatefulStoragePallet::apply_item_actions(
			RuntimeOrigin::signed(caller_1),
			msa_id,
			schema_id,
			NONEXISTENT_PAGE_HASH,
			BoundedVec::try_from(actions1).unwrap(),
		));
	});
}

#[test]
fn apply_item_actions_existing_page_with_stale_hash_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let msa_id = 1;
		let caller_1 = test_public(msa_id);
		let schema_id = ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let actions1 = vec![ItemAction::Add { data: payload.clone().try_into().unwrap() }];

		let page = ItemizedPage::<Test>::default();
		let page_hash = page.get_hash();
		let mut new_page =
			ItemizedOperations::<Test>::apply_item_actions(&page, &actions1).unwrap();
		new_page.nonce = 1;
		let key = (schema_id,);
		<StatefulChildTree>::write(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			ITEMIZED_STORAGE_PREFIX,
			&key,
			&new_page,
		);

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				page_hash,
				BoundedVec::try_from(actions1).unwrap(),
			),
			Error::<Test>::StalePageState
		)
	});
}

#[test]
fn apply_delete_item_on_append_only_fails() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let msa_id = 1;
		let schema_id = ITEMIZED_APPEND_ONLY_SCHEMA;
		let payload = vec![1; 5];
		let actions1 = vec![ItemAction::Add { data: payload.try_into().unwrap() }];
		let actions2 = vec![ItemAction::Delete { index: 0 }];
		let keys = (schema_id,);
		assert_ok!(StatefulStoragePallet::apply_item_actions(
			RuntimeOrigin::signed(caller_1.clone()),
			msa_id,
			schema_id,
			NONEXISTENT_PAGE_HASH,
			BoundedVec::try_from(actions1).unwrap(),
		));

		let items1: Option<ItemizedPage<Test>> =
			StatefulChildTree::<<Test as Config>::KeyHasher>::try_read(
				&msa_id,
				PALLET_STORAGE_PREFIX,
				ITEMIZED_STORAGE_PREFIX,
				&keys,
			)
			.unwrap();
		assert!(items1.is_some());
		let content_hash = items1.unwrap().get_hash();

		// assert
		assert_err!(
			StatefulStoragePallet::apply_item_actions(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				content_hash,
				BoundedVec::try_from(actions2).unwrap(),
			),
			Error::<Test>::UnsupportedOperationForSchema
		);
	});
}

#[test]
fn apply_item_actions_with_signature_v2_having_wrong_signature_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (_, pair) = get_signature_account();
		let delegator_key = pair.public();
		let (signature_key, _) = sr25519::Pair::generate();
		let schema_id = ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let actions = vec![ItemAction::Add { data: payload.try_into().unwrap() }];
		let payload = ItemizedSignaturePayloadV2 {
			actions: BoundedVec::try_from(actions).unwrap(),
			target_hash: PageHash::default(),
			expiration: 10,
			schema_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = signature_key.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions_with_signature_v2(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::InvalidSignature
		)
	});
}

#[test]
fn apply_item_actions_with_signature_v2_having_too_far_expiration_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (_, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let actions = vec![ItemAction::Add { data: payload.try_into().unwrap() }];
		let mortality_window: u32 = <Test as Config>::MortalityWindowSize::get();
		let payload = ItemizedSignaturePayloadV2 {
			actions: BoundedVec::try_from(actions).unwrap(),
			target_hash: PageHash::default(),
			expiration: (mortality_window + 1).into(),
			schema_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions_with_signature_v2(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::ProofNotYetValid
		)
	});
}

#[test]
fn apply_item_actions_with_signature_v2_having_expired_payload_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (_, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let actions = vec![ItemAction::Add { data: payload.try_into().unwrap() }];
		let block_number = 10;
		let payload = ItemizedSignaturePayloadV2 {
			actions: BoundedVec::try_from(actions).unwrap(),
			target_hash: PageHash::default(),
			expiration: block_number,
			schema_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		System::set_block_number(block_number);
		assert_err!(
			StatefulStoragePallet::apply_item_actions_with_signature_v2(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::ProofHasExpired
		)
	});
}

#[test]
fn apply_item_actions_with_signature_v2_having_correct_input_should_work() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = ITEMIZED_SCHEMA;
		let prev_content_hash = PageHash::default();
		let payload = vec![1; 5];
		let actions = vec![ItemAction::Add { data: payload.try_into().unwrap() }];
		let payload = ItemizedSignaturePayloadV2 {
			actions: BoundedVec::try_from(actions).unwrap(),
			target_hash: prev_content_hash,
			expiration: 10,
			schema_id,
		};

		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_ok!(StatefulStoragePallet::apply_item_actions_with_signature_v2(
			RuntimeOrigin::signed(caller_1),
			delegator_key.into(),
			owner_signature,
			payload
		));

		// assert
		let updated_page: Option<ItemizedPage<Test>> =
			StatefulChildTree::<<Test as Config>::KeyHasher>::try_read(
				&msa_id,
				PALLET_STORAGE_PREFIX,
				ITEMIZED_STORAGE_PREFIX,
				&(schema_id,),
			)
			.unwrap();
		assert!(updated_page.is_some());
		let curr_content_hash = updated_page.unwrap().get_hash();
		System::assert_last_event(
			StatefulEvent::ItemizedPageUpdated {
				msa_id,
				schema_id,
				prev_content_hash,
				curr_content_hash,
			}
			.into(),
		);
	});
}

#[test]
fn apply_item_actions_with_signature_v2_having_non_existing_msa_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let pair = get_invalid_msa_signature_account(); // hardcoded key that returns None Msa
		let delegator_key = pair.public();
		let schema_id = ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let actions = vec![ItemAction::Add { data: payload.try_into().unwrap() }];
		let payload = ItemizedSignaturePayloadV2 {
			actions: BoundedVec::try_from(actions).unwrap(),
			target_hash: PageHash::default(),
			expiration: 10,
			schema_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions_with_signature_v2(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::InvalidMessageSourceAccount
		)
	});
}

#[test]
fn apply_item_actions_with_signature_v2_having_invalid_schema_id_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (_, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = INVALID_SCHEMA_ID;
		let payload = vec![1; 5];
		let actions = vec![ItemAction::Add { data: payload.try_into().unwrap() }];
		let payload = ItemizedSignaturePayloadV2 {
			actions: BoundedVec::try_from(actions).unwrap(),
			target_hash: PageHash::default(),
			expiration: 10,
			schema_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions_with_signature_v2(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::InvalidSchemaId
		)
	});
}

#[test]
fn apply_item_actions_with_signature_v2_having_invalid_schema_location_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (_, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = PAGINATED_SCHEMA;
		let payload = vec![1; 5];
		let actions = vec![ItemAction::Add { data: payload.try_into().unwrap() }];
		let payload = ItemizedSignaturePayloadV2 {
			actions: BoundedVec::try_from(actions).unwrap(),
			target_hash: PageHash::default(),
			expiration: 10,
			schema_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions_with_signature_v2(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::SchemaPayloadLocationMismatch
		)
	});
}

#[test]
fn apply_item_actions_with_signature_v2_having_valid_input_and_empty_items_should_remove_storage() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let actions1 = vec![ItemAction::Add { data: payload.try_into().unwrap() }];
		let actions2 = vec![ItemAction::Delete { index: 0 }];
		let keys = (schema_id,);
		assert_ok!(StatefulStoragePallet::apply_item_actions(
			RuntimeOrigin::signed(caller_1.clone()),
			msa_id,
			schema_id,
			NONEXISTENT_PAGE_HASH,
			BoundedVec::try_from(actions1).unwrap(),
		));

		let items1: Option<ItemizedPage<Test>> =
			StatefulChildTree::<<Test as Config>::KeyHasher>::try_read(
				&msa_id,
				PALLET_STORAGE_PREFIX,
				ITEMIZED_STORAGE_PREFIX,
				&keys,
			)
			.unwrap();
		assert!(items1.is_some());
		let content_hash = items1.unwrap().get_hash();

		let payload = ItemizedSignaturePayloadV2 {
			actions: BoundedVec::try_from(actions2).unwrap(),
			target_hash: content_hash,
			expiration: 10,
			schema_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_ok!(StatefulStoragePallet::apply_item_actions_with_signature_v2(
			RuntimeOrigin::signed(caller_1),
			delegator_key.into(),
			owner_signature,
			payload
		));

		// assert
		let items2: Option<ItemizedPage<Test>> =
			StatefulChildTree::<<Test as Config>::KeyHasher>::try_read(
				&msa_id,
				PALLET_STORAGE_PREFIX,
				ITEMIZED_STORAGE_PREFIX,
				&(schema_id,),
			)
			.unwrap();
		assert!(items2.is_none());
		System::assert_last_event(
			StatefulEvent::ItemizedPageDeleted {
				msa_id,
				schema_id,
				prev_content_hash: content_hash,
			}
			.into(),
		);
	});
}

#[test]
fn apply_item_actions_with_signature_v2_having_corrupted_state_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let page: ItemizedPage<Test> = generate_page(None, None);
		let actions = vec![ItemAction::Add { data: payload.clone().try_into().unwrap() }];
		let key = (schema_id,);
		StatefulChildTree::<<Test as Config>::KeyHasher>::write(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			ITEMIZED_STORAGE_PREFIX,
			&key,
			&page,
		);
		let previous_page: ItemizedPage<Test> =
			StatefulChildTree::<<Test as Config>::KeyHasher>::try_read(
				&msa_id,
				PALLET_STORAGE_PREFIX,
				ITEMIZED_STORAGE_PREFIX,
				&key,
			)
			.unwrap()
			.unwrap();
		let previous_hash = previous_page.get_hash();

		let payload = ItemizedSignaturePayloadV2 {
			actions: BoundedVec::try_from(actions).unwrap(),
			target_hash: previous_hash,
			expiration: 10,
			schema_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions_with_signature_v2(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::CorruptedState
		)
	});
}

#[test]
fn apply_item_actions_with_signature_v2_having_page_with_stale_hash_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = ITEMIZED_SCHEMA;
		let payload = vec![1; 5];
		let actions = vec![ItemAction::Add { data: payload.clone().try_into().unwrap() }];
		let page = ItemizedPage::<Test>::default();
		let page_hash = page.get_hash();
		let page = ItemizedOperations::<Test>::apply_item_actions(&page, &actions).unwrap();
		let key = (schema_id,);
		<StatefulChildTree>::write(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			ITEMIZED_STORAGE_PREFIX,
			&key,
			page,
		);
		let payload = ItemizedSignaturePayloadV2 {
			actions: BoundedVec::try_from(actions).unwrap(),
			target_hash: page_hash,
			expiration: 10,
			schema_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let owner_signature: MultiSignature = pair.sign(&encode_data_new_key_data).into();

		// act
		assert_err!(
			StatefulStoragePallet::apply_item_actions_with_signature_v2(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_signature,
				payload
			),
			Error::<Test>::StalePageState
		)
	});
}
