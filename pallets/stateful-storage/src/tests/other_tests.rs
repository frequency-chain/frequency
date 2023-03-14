use crate::{
	stateful_child_tree::StatefulChildTree,
	tests::{
		mock::*,
		test_common::{constants::*, test_constants::*},
	},
	types::*,
	Config, Error,
};
use codec::Encode;
use common_primitives::utils::wrap_binary_data;
use frame_support::{assert_err, assert_ok, BoundedVec};
#[allow(unused_imports)]
use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};
use sp_core::Pair;
use sp_runtime::MultiSignature;

#[test]
fn is_empty_false_for_non_empty_page() {
	let page: ItemizedPage<Test> =
		create_itemized_page_from::<Test>(None, &[generate_payload_bytes(None)]);

	assert_eq!(page.is_empty(), false);
}

#[test]
fn is_empty_true_for_empty_page() {
	let page: ItemizedPage<Test> = create_itemized_page_from::<Test>(None, &[]);

	assert_eq!(page.is_empty(), true);
}

#[test]
fn apply_actions_on_signature_schema_fails() {
	new_test_ext().execute_with(|| {
		// arrange
		let caller_1 = test_public(1);
		let msa_id = 1;
		let schema_id = ITEMIZED_SIGNATURE_REQUIRED_SCHEMA;
		let payload = vec![1; 5];
		let actions1 = vec![ItemAction::Add { data: payload.try_into().unwrap() }];
		assert_err!(
			StatefulStoragePallet::apply_item_actions(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				NONEXISTENT_PAGE_HASH,
				BoundedVec::try_from(actions1).unwrap(),
			),
			Error::<Test>::UnsupportedOperationForSchema
		);
	});
}

#[test]
fn insert_page_fails_for_signature_schema() {
	new_test_ext().execute_with(|| {
		// setup
		let caller_1 = test_public(1);
		let msa_id = 1;
		let schema_id = PAGINATED_SIGNED_SCHEMA;
		let page_id = 11;
		let payload = generate_payload_bytes::<PaginatedPageSize>(None);

		// assert
		assert_err!(
			StatefulStoragePallet::upsert_page(
				RuntimeOrigin::signed(caller_1),
				msa_id,
				schema_id,
				page_id,
				NONEXISTENT_PAGE_HASH,
				payload.into(),
			),
			Error::<Test>::UnsupportedOperationForSchema
		);
	});
}

#[test]
fn signature_replay_on_existing_page_errors() {
	new_test_ext().execute_with(|| {
		// Setup
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 1;
		let keys = (schema_id, page_id);
		let page_a: PaginatedPage<Test> = generate_page(Some(1), Some(1));
		let page_b: PaginatedPage<Test> = generate_page(Some(2), Some(2));
		let payload_a_to_b = PaginatedUpsertSignaturePayload {
			msa_id,
			expiration: 10,
			schema_id,
			page_id,
			target_hash: page_a.get_hash(),
			payload: page_b.data.clone(),
		};

		// Set up initial state A
		<StatefulChildTree>::write(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&keys,
			&page_a,
		);

		// Make sure we successfully apply state transition A -> B
		let encoded_payload = wrap_binary_data(payload_a_to_b.encode());
		let owner_a_to_b_signature: MultiSignature = pair.sign(&encoded_payload).into();
		assert_ok!(StatefulStoragePallet::upsert_page_with_signature(
			RuntimeOrigin::signed(caller_1.clone()),
			delegator_key.into(),
			owner_a_to_b_signature.clone(),
			payload_a_to_b.clone()
		));

		// Read back page state & get hash
		let current_page: PaginatedPage<Test> =
			StatefulChildTree::<<Test as Config>::KeyHasher>::try_read(
				&msa_id,
				PALLET_STORAGE_PREFIX,
				PAGINATED_STORAGE_PREFIX,
				&keys,
			)
			.unwrap()
			.expect("no page read");

		// Make sure we successfully apply state transition B -> A
		assert_ok!(StatefulStoragePallet::upsert_page(
			RuntimeOrigin::signed(caller_1.clone()),
			msa_id,
			schema_id,
			page_id,
			current_page.get_hash(),
			page_a.data
		));

		// Signature replay A -> B should fail
		assert_err!(
			StatefulStoragePallet::upsert_page_with_signature(
				RuntimeOrigin::signed(caller_1),
				delegator_key.into(),
				owner_a_to_b_signature,
				payload_a_to_b
			),
			Error::<Test>::StalePageState
		);
	})
}

// NOTE: This is a known issue. When it's fixed, this test will start failing & we can change the test assertion.
#[test]
fn signature_replay_on_deleted_page_check() {
	new_test_ext().execute_with(|| {
		// Setup
		let caller_1 = test_public(1);
		let (msa_id, pair) = get_signature_account();
		let delegator_key = pair.public();
		let schema_id = PAGINATED_SCHEMA;
		let page_id = 1;
		let keys = (schema_id, page_id);
		let page_a: PaginatedPage<Test> = generate_page(Some(1), Some(1));
		let payload_null_to_a = PaginatedUpsertSignaturePayload {
			msa_id,
			expiration: 10,
			schema_id,
			page_id,
			target_hash: NONEXISTENT_PAGE_HASH,
			payload: page_a.data.clone(),
		};

		// Make sure we successfully apply state transition Null -> A
		let encoded_payload = wrap_binary_data(payload_null_to_a.encode());
		let owner_null_to_a_signature: MultiSignature = pair.sign(&encoded_payload).into();
		assert_ok!(StatefulStoragePallet::upsert_page_with_signature(
			RuntimeOrigin::signed(caller_1.clone()),
			delegator_key.into(),
			owner_null_to_a_signature.clone(),
			payload_null_to_a.clone()
		));

		// Read back page state & get hash
		let current_page: PaginatedPage<Test> =
			StatefulChildTree::<<Test as Config>::KeyHasher>::try_read(
				&msa_id,
				PALLET_STORAGE_PREFIX,
				PAGINATED_STORAGE_PREFIX,
				&keys,
			)
			.unwrap()
			.expect("no page read");

		// Make sure we successfully delete the page
		assert_ok!(StatefulStoragePallet::delete_page(
			RuntimeOrigin::signed(caller_1.clone()),
			msa_id,
			schema_id,
			page_id,
			current_page.get_hash(),
		));

		// Signature replay A -> B (change assertion when this vulnerability is fixed)
		assert_ok!(StatefulStoragePallet::upsert_page_with_signature(
			RuntimeOrigin::signed(caller_1.clone()),
			delegator_key.into(),
			owner_null_to_a_signature,
			payload_null_to_a
		));
	})
}
