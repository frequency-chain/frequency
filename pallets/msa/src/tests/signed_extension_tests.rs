use crate::{tests::mock::*, types::EMPTY_FUNCTION, CheckFreeExtrinsicUse, ValidityError};
use common_runtime::extensions::check_nonce::CheckNonce;
use frame_support::{
	assert_err, assert_ok,
	dispatch::{DispatchInfo, GetDispatchInfo},
	pallet_prelude::InvalidTransaction,
};
use sp_core::{crypto::AccountId32, sr25519, sr25519::Public, Pair};
#[allow(deprecated)]
use sp_runtime::{traits::SignedExtension, transaction_validity::TransactionValidity};

// Assert that CheckFreeExtrinsicUse::validate fails with `expected_err_enum`,
// for the "delete_msa_public_key" call, given extrinsic caller = caller_key,
// when attempting to delete `public_key_to_delete`
fn assert_validate_key_delete_fails(
	caller_key: &AccountId32,
	public_key_to_delete: AccountId32,
	expected_err_enum: ValidityError,
) {
	let call_delete_msa_public_key: &<Test as frame_system::Config>::RuntimeCall =
		&RuntimeCall::Msa(MsaCall::delete_msa_public_key { public_key_to_delete });

	let expected_err: TransactionValidity =
		InvalidTransaction::Custom(expected_err_enum as u8).into();

	assert_eq!(
		CheckFreeExtrinsicUse::<Test>::new().validate(
			&caller_key,
			call_delete_msa_public_key,
			&DispatchInfo::default(),
			0_usize,
		),
		expected_err
	);
}

fn assert_revoke_delegation_by_provider_err(
	expected_err: InvalidTransaction,
	provider_account: Public,
	delegator_msa_id: u64,
) {
	let call_revoke_delegation: &<Test as frame_system::Config>::RuntimeCall =
		&RuntimeCall::Msa(MsaCall::revoke_delegation_by_provider { delegator: delegator_msa_id });
	let info = DispatchInfo::default();
	let len = 0_usize;
	let result = CheckFreeExtrinsicUse::<Test>::new().validate(
		&provider_account.into(),
		call_revoke_delegation,
		&info,
		len,
	);
	assert_err!(result, expected_err);
}

/// Assert that revoking an MSA delegation passes the signed extension CheckFreeExtrinsicUse
/// validation when a valid delegation exists.
#[test]
fn signed_extension_revoke_delegation_by_delegator_success() {
	new_test_ext().execute_with(|| {
		let (provider_msa_id, delegator_account) = create_provider_msa_and_delegator();
		let call_revoke_delegation: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::revoke_delegation_by_delegator { provider_msa_id });
		let info = DispatchInfo::default();
		let len = 0_usize;
		let result = CheckFreeExtrinsicUse::<Test>::new().validate(
			&delegator_account.into(),
			call_revoke_delegation,
			&info,
			len,
		);
		assert_ok!(result);
	});
}

/// Assert that revoking an MSA delegation fails the signed extension CheckFreeExtrinsicUse
/// validation when no valid delegation exists.
#[test]
fn signed_extension_fails_when_revoke_delegation_by_delegator_called_twice() {
	new_test_ext().execute_with(|| {
		let (provider_msa_id, delegator_account) = create_provider_msa_and_delegator();
		let call_revoke_delegation: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::revoke_delegation_by_delegator { provider_msa_id });
		let info = DispatchInfo::default();
		let len = 0_usize;
		let result = CheckFreeExtrinsicUse::<Test>::new().validate(
			&delegator_account.into(),
			call_revoke_delegation,
			&info,
			len,
		);
		assert_ok!(result);
		assert_ok!(Msa::revoke_delegation_by_delegator(
			RuntimeOrigin::signed(delegator_account.into()),
			provider_msa_id
		));

		System::set_block_number(System::block_number() + 1);
		let call_revoke_delegation: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::revoke_delegation_by_delegator { provider_msa_id });
		let info = DispatchInfo::default();
		let len = 0_usize;
		let result_revoked = CheckFreeExtrinsicUse::<Test>::new().validate(
			&delegator_account.into(),
			call_revoke_delegation,
			&info,
			len,
		);
		assert!(result_revoked.is_err());
	});
}

#[test]
fn signed_extension_revoke_delegation_by_provider_success() {
	new_test_ext().execute_with(|| {
		let (delegator_msa_id, provider_account) = create_delegator_msa_and_provider();
		let call_revoke_delegation: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::revoke_delegation_by_provider {
				delegator: delegator_msa_id,
			});
		let info = DispatchInfo::default();
		let len = 0_usize;
		let result = CheckFreeExtrinsicUse::<Test>::new().validate(
			&provider_account.into(),
			call_revoke_delegation,
			&info,
			len,
		);
		assert_ok!(result);
	})
}

#[test]
fn signed_extension_revoke_delegation_by_provider_fails_when_no_delegator_msa() {
	new_test_ext().execute_with(|| {
		let (_, provider_pair) = create_account();
		let provider_account = provider_pair.public();

		let delegator_msa_id = 33u64;
		let expected_err = InvalidTransaction::Custom(ValidityError::InvalidDelegation as u8);
		assert_revoke_delegation_by_provider_err(expected_err, provider_account, delegator_msa_id);
	})
}

#[test]
fn signed_extension_revoke_delegation_by_provider_fails_when_no_provider_msa() {
	new_test_ext().execute_with(|| {
		let (provider_pair, _) = sr25519::Pair::generate();
		let provider_account = provider_pair.public();

		let (delegator_msa, _) = create_account();

		let expected_err = InvalidTransaction::Custom(ValidityError::InvalidMsaKey as u8);
		assert_revoke_delegation_by_provider_err(expected_err, provider_account, delegator_msa);
	});
}

#[test]
fn signed_extension_revoke_delegation_by_provider_fails_when_no_delegation() {
	new_test_ext().execute_with(|| {
		let (_, provider_pair) = create_account();
		let provider_account = provider_pair.public();
		let (delegator_msa, _) = create_account();

		let expected_err = InvalidTransaction::Custom(ValidityError::InvalidDelegation as u8);
		assert_revoke_delegation_by_provider_err(expected_err, provider_account, delegator_msa);
	})
}

/// Assert that a call that is not one of the matches passes the signed extension
/// CheckFreeExtrinsicUse validation.
#[test]
fn signed_extension_validation_valid_for_other_extrinsics() {
	let random_call_should_pass: &<Test as frame_system::Config>::RuntimeCall =
		&RuntimeCall::Msa(MsaCall::create {});
	let info = DispatchInfo::default();
	let len = 0_usize;
	let result = CheckFreeExtrinsicUse::<Test>::new().validate(
		&test_public(1),
		random_call_should_pass,
		&info,
		len,
	);
	assert_ok!(result);
}

// Assert that check nonce validation does not create a token account for delete_msa_public_key call.
#[test]
fn signed_ext_check_nonce_delete_msa_public_key() {
	new_test_ext().execute_with(|| {
		// Generate a key pair for MSA account
		let (msa_key_pair, _) = sr25519::Pair::generate();
		let msa_new_key = msa_key_pair.public();

		let len = 0_usize;

		// Test the delete_msa_public_key() call
		let call_delete_msa_public_key: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::delete_msa_public_key {
				public_key_to_delete: AccountId32::from(msa_new_key),
			});
		let info = call_delete_msa_public_key.get_dispatch_info();

		// Call delete_msa_public_key() using the Alice account
		let who = test_public(1);
		assert_ok!(CheckNonce::<Test>(0).pre_dispatch(
			&who,
			call_delete_msa_public_key,
			&info,
			len
		));

		// Did the call create a token account?
		let created_token_account: bool;
		match frame_system::Account::<Test>::try_get(who) {
			Ok(_) => {
				created_token_account = true;
			},
			Err(_) => {
				created_token_account = false;
			},
		};

		// Assert that the call did not create a token account
		assert_eq!(created_token_account, false);
	})
}

// Assert that check nonce validation does not create a token account for revoke_delegation_by_delegator call.
#[test]
fn signed_ext_check_nonce_revoke_delegation_by_delegator() {
	new_test_ext().execute_with(|| {
		let (provider_msa_id, _) = create_provider_msa_and_delegator();

		// We are testing the revoke_delegation_by_delegator() call.
		let call_revoke_delegation_by_delegator: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::revoke_delegation_by_delegator { provider_msa_id });

		let len = 0_usize;

		// Get the dispatch info for the call.
		let info = call_revoke_delegation_by_delegator.get_dispatch_info();

		// Call revoke_delegation_by_delegator() using the Alice account
		let who = test_public(1);
		assert_ok!(CheckNonce::<Test>(0).pre_dispatch(
			&who,
			call_revoke_delegation_by_delegator,
			&info,
			len
		));

		// Did the call create a token account?
		let created_token_account: bool;
		match frame_system::Account::<Test>::try_get(who) {
			Ok(_) => {
				created_token_account = true;
			},
			Err(_) => {
				created_token_account = false;
			},
		};

		// Assert that the call did not create a token account
		assert_eq!(created_token_account, false);
	})
}

// Assert that check nonce validation does create a token account for a paying call.
#[test]
fn signed_ext_check_nonce_creates_token_account_if_paying() {
	new_test_ext().execute_with(|| {
		//  Test that a  "pays" extrinsic creates a token account
		let who = test_public(1);
		let len = 0_usize;
		let pays_call_should_pass: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::create {});

		// Get the dispatch info for the create() call.
		let pays_call_should_pass_info = pays_call_should_pass.get_dispatch_info();

		// Call create() using the Alice account
		assert_ok!(CheckNonce::<Test>(0).pre_dispatch(
			&who,
			pays_call_should_pass,
			&pays_call_should_pass_info,
			len
		));

		// Did the call create a token account?
		let created_token_account: bool;
		match frame_system::Account::<Test>::try_get(who) {
			Ok(_) => {
				created_token_account = true;
			},
			Err(_) => {
				created_token_account = false;
			},
		};
		// Assert that the call created a token account
		assert_eq!(created_token_account, true);
	})
}

#[test]
#[allow(deprecated)]
fn signed_ext_check_nonce_increases_nonce_for_an_existing_account_for_free_transactions() {
	new_test_ext().execute_with(|| {
		// arrange
		let who = test_public(1);
		let len = 0_usize;
		let free_call: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::delete_msa_public_key { public_key_to_delete: who.clone() });
		let free_call_info = free_call.get_dispatch_info();
		let mut account = frame_system::Account::<Test>::get(who.clone());
		account.consumers += 1;
		frame_system::Account::<Test>::insert(who.clone(), account);

		// act
		assert_ok!(CheckNonce::<Test>(0).pre_dispatch(
			&who.clone(),
			free_call,
			&free_call_info,
			len
		));

		// assert
		let account_after = frame_system::Account::<Test>::try_get(who).expect("should resolve");
		assert_eq!(account_after.nonce, 1);
	})
}

#[test]
fn signed_extension_validation_delete_msa_public_key_success() {
	new_test_ext().execute_with(|| {
		let (msa_id, original_key_pair) = create_account();

		let (new_key_pair, _) = sr25519::Pair::generate();
		let new_key: AccountId32 = new_key_pair.public().into();
		assert_ok!(Msa::add_key(msa_id, &new_key, EMPTY_FUNCTION));

		let original_key: AccountId32 = original_key_pair.public().into();

		// set up call for new key to delete original key
		let call_delete_msa_public_key: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::delete_msa_public_key {
				public_key_to_delete: original_key.clone(),
			});

		let info = DispatchInfo::default();
		let len = 0_usize;
		assert_ok!(CheckFreeExtrinsicUse::<Test>::new().validate(
			&new_key,
			call_delete_msa_public_key,
			&info,
			len,
		));

		// validate other direction
		let call_delete_msa_public_key2: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::delete_msa_public_key { public_key_to_delete: new_key });
		assert_ok!(CheckFreeExtrinsicUse::<Test>::new().validate(
			&original_key,
			call_delete_msa_public_key2,
			&info,
			len,
		));
	});
}

#[test]
#[allow(deprecated)]
fn signed_extension_validate_fails_when_delete_msa_public_key_called_twice() {
	new_test_ext().execute_with(|| {
		let (owner_msa_id, owner_key_pair) = create_account();

		let (new_key_pair, _) = sr25519::Pair::generate();
		let new_key: AccountId32 = new_key_pair.public().into();
		assert_ok!(Msa::add_key(owner_msa_id, &new_key, EMPTY_FUNCTION));

		let call_delete_msa_public_key: &<Test as frame_system::Config>::RuntimeCall =
			&RuntimeCall::Msa(MsaCall::delete_msa_public_key {
				public_key_to_delete: owner_key_pair.public().into(),
			});

		// check that it's okay to delete the original key
		assert_ok!(CheckFreeExtrinsicUse::<Test>::new().validate(
			&new_key,
			call_delete_msa_public_key,
			&DispatchInfo::default(),
			0_usize,
		));

		// new key deletes the old key
		assert_ok!(Msa::delete_msa_public_key(
			RuntimeOrigin::signed(new_key.clone()),
			owner_key_pair.public().into()
		));

		assert_validate_key_delete_fails(
			&new_key,
			owner_key_pair.public().into(),
			ValidityError::InvalidMsaKey,
		);
	});
}

#[test]
fn signed_extension_validate_fails_when_delete_msa_public_key_called_on_only_key() {
	new_test_ext().execute_with(|| {
		let (_, original_pair) = create_account();
		let original_key: AccountId32 = original_pair.public().into();

		assert_validate_key_delete_fails(
			&original_key,
			original_key.clone(),
			ValidityError::InvalidSelfRemoval,
		)
	})
}

#[test]
fn signed_extension_validate_fails_when_delete_msa_public_key_called_by_non_owner() {
	new_test_ext().execute_with(|| {
		let (_, original_pair) = create_account();
		let original_key: AccountId32 = original_pair.public().into();

		let (_, non_owner_pair) = create_account();
		let non_owner_key: AccountId32 = non_owner_pair.public().into();
		assert_validate_key_delete_fails(
			&non_owner_key,
			original_key.clone(),
			ValidityError::NotKeyOwner,
		)
	})
}
