use frame_support::{assert_err, assert_noop, assert_ok, pallet_prelude::InvalidTransaction};

use sp_core::{crypto::AccountId32, sr25519, Encode, Pair};
use sp_runtime::MultiSignature;

use crate::{
	tests::mock::*,
	types::{AddKeyData, EMPTY_FUNCTION},
	CheckFreeExtrinsicUse, Error, Event, ValidityError,
};

use crate::tests::other_tests::{
	assert_revoke_delegation_by_delegator_no_effect, set_schema_count,
};
use common_primitives::{
	handles::ClaimHandlePayload,
	msa::{DelegatorId, ProviderId},
	utils::wrap_binary_data,
};

#[test]
fn test_retire_msa_success() {
	new_test_ext().execute_with(|| {
		let (test_account_key_pair, _) = sr25519::Pair::generate();

		// Create an account
		let test_account = AccountId32::new(test_account_key_pair.public().into());
		let origin = RuntimeOrigin::signed(test_account.clone());

		// Create an MSA so this account has one key associated with it
		assert_ok!(Msa::create(origin.clone()));
		let msa_id = Msa::get_owner_of(&test_account).unwrap();

		// Retire the MSA
		assert_ok!(Msa::retire_msa(origin));

		// Check if PublicKeyDeleted event was dispatched.
		System::assert_has_event(Event::PublicKeyDeleted { key: test_account.clone() }.into());

		// Check if MsaRetired event was dispatched.
		System::assert_last_event(Event::MsaRetired { msa_id }.into());

		// Assert that the MSA has no accounts
		let key_count = Msa::get_public_key_count_by_msa_id(msa_id);
		assert_eq!(key_count, 0);

		// MSA has been retired, perform additional tests

		// [TEST] Adding an account to the retired MSA should fail
		let (key_pair1, _) = sr25519::Pair::generate();
		let new_account1 = key_pair1.public();
		let (msa_id2, _) = create_account();

		let add_new_key_data =
			AddKeyData { msa_id: msa_id2, expiration: 10, new_public_key: new_account1.into() };

		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());
		let old_msa_owner_signature: MultiSignature =
			test_account_key_pair.sign(&encode_data_new_key_data).into();
		let new_key_signature: MultiSignature = key_pair1.sign(&encode_data_new_key_data).into();
		assert_noop!(
			Msa::add_public_key_to_msa(
				RuntimeOrigin::signed(test_account.clone()),
				test_account_key_pair.public().into(),
				old_msa_owner_signature.clone(),
				new_key_signature,
				add_new_key_data
			),
			Error::<Test>::NoKeyExists
		);

		// [TEST] Adding a provider to the retired MSA should fail
		let (provider_key_pair, _) = sr25519::Pair::generate();
		let provider_account = provider_key_pair.public();

		// Create provider account and get its MSA ID (u64)
		assert_ok!(Msa::create(RuntimeOrigin::signed(provider_account.into())));
		let provider_msa_id =
			Msa::ensure_valid_msa_key(&AccountId32::new(provider_account.0)).unwrap();

		assert_ok!(Msa::create_provider(
			RuntimeOrigin::signed(provider_account.into()),
			Vec::from("Foo")
		));

		let (delegator_signature, add_provider_payload) =
			create_and_sign_add_provider_payload(test_account_key_pair, provider_msa_id);

		assert_noop!(
			Msa::grant_delegation(
				RuntimeOrigin::signed(provider_account.into()),
				test_account.clone(),
				delegator_signature,
				add_provider_payload
			),
			Error::<Test>::NoKeyExists
		);

		// [TEST] Revoking a delegation (modifying permissions) should not do anything
		assert_revoke_delegation_by_delegator_no_effect(test_account, provider_msa_id)
	})
}

#[test]
fn test_retire_msa_does_nothing_when_no_msa() {
	new_test_ext().execute_with(|| {
		let (test_pair, _) = sr25519::Pair::generate();
		let first_account_key = test_pair.public();
		let origin = RuntimeOrigin::signed(first_account_key.into());

		// 1. when there's no MSA at all
		let event_count = System::event_count();
		assert_ok!(Msa::retire_msa(origin.clone()));
		assert_eq!(event_count, System::event_count());
	});
}

#[test]
fn test_ensure_msa_can_retire_fails_if_registered_provider() {
	new_test_ext().execute_with(|| {
		// Create an account
		let (test_account_key_pair, _) = sr25519::Pair::generate();
		let test_account = AccountId32::new(test_account_key_pair.public().into());
		let origin = RuntimeOrigin::signed(test_account.clone());

		// Add an account to the MSA
		assert_ok!(Msa::add_key(2, &test_account, EMPTY_FUNCTION));

		// Register provider
		assert_ok!(Msa::create_provider(origin, Vec::from("Foo")));

		// Retire MSA
		assert_noop!(
			CheckFreeExtrinsicUse::<Test>::ensure_msa_can_retire(&test_account),
			InvalidTransaction::Custom(
				ValidityError::InvalidRegisteredProviderCannotBeRetired as u8
			)
		);
	})
}

#[test]
fn test_ensure_msa_can_retire_fails_if_more_than_one_account_exists() {
	new_test_ext().execute_with(|| {
		let msa_id = 2;
		let (test_account_1_key_pair, _) = sr25519::Pair::generate();
		let (test_account_2_key_pair, _) = sr25519::Pair::generate();

		// Create accounts
		let test_account_1 = AccountId32::new(test_account_1_key_pair.public().into());
		let test_account_2 = AccountId32::new(test_account_2_key_pair.public().into());

		// Add two accounts to the MSA
		assert_ok!(Msa::add_key(msa_id, &test_account_1, EMPTY_FUNCTION));
		assert_ok!(Msa::add_key(msa_id, &test_account_2, EMPTY_FUNCTION));

		// Retire the MSA
		assert_noop!(
			CheckFreeExtrinsicUse::<Test>::ensure_msa_can_retire(&test_account_1),
			InvalidTransaction::Custom(ValidityError::InvalidMoreThanOneKeyExists as u8)
		);
	})
}

#[test]
fn test_ensure_msa_can_retire_fails_if_any_active_delegations_exist() {
	new_test_ext().execute_with(|| {
		// Create delegator
		let msa_id = 2;
		let (test_account_key_pair, _) = sr25519::Pair::generate();
		let test_account = AccountId32::new(test_account_key_pair.public().into());
		assert_ok!(Msa::add_key(msa_id, &test_account, EMPTY_FUNCTION));

		// Create provider
		let (provider_id, _provider_key) = create_provider_with_name("test");
		let schema_ids = vec![1];
		set_schema_count::<Test>(1);
		assert_ok!(Msa::add_provider(ProviderId(provider_id), DelegatorId(msa_id), schema_ids));

		// Retire the MSA
		assert_err!(
			CheckFreeExtrinsicUse::<Test>::ensure_msa_can_retire(&test_account),
			InvalidTransaction::Custom(ValidityError::InvalidNonZeroProviderDelegations as u8)
		);
	})
}

#[test]
fn test_ensure_msa_cannot_retire_if_handle_exists() {
	new_test_ext().execute_with(|| {
		let msa_id = 1;
		let (test_account_1_key_pair, _) = sr25519::Pair::generate();

		// Create accounts
		let test_account_1 = AccountId32::new(test_account_1_key_pair.public().into());

		// Add two accounts to the MSA
		assert_ok!(Msa::add_key(msa_id, &test_account_1, EMPTY_FUNCTION));

		let claim_payload = ClaimHandlePayload::<<Test as frame_system::Config>::BlockNumber> {
			base_handle: "hello".into(),
			expiration: 2,
		};

		assert_ok!(pallet_handles::Pallet::<Test>::do_claim_handle(msa_id, claim_payload));

		// Assumption: handle exists
		// Retire the MSA
		assert_noop!(
			CheckFreeExtrinsicUse::<Test>::ensure_msa_can_retire(&test_account_1),
			InvalidTransaction::Custom(ValidityError::HandleNotRetired as u8)
		);
	})
}
