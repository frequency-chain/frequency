use frame_support::{
	assert_err, assert_noop, assert_ok,
	pallet_prelude::InvalidTransaction,
	weights::{DispatchInfo, GetDispatchInfo, Pays, Weight},
	BoundedBTreeMap,
};
use sp_core::{crypto::AccountId32, sr25519, sr25519::Public, Encode, Pair};
use sp_runtime::{traits::SignedExtension, MultiSignature};

use crate::{
	ensure,
	mock::*,
	types::{AddKeyData, AddProvider, PermittedDelegationSchemas, EMPTY_FUNCTION},
	CheckFreeExtrinsicUse, Config, CurrentMsaIdentifierMaximum, DispatchResult, Error, Event,
	PayloadSignatureRegistry, ProviderToRegistryEntry, ValidityError,
};

use common_primitives::{
	msa::{Delegation, Delegator, MessageSourceId, Provider, ProviderRegistryEntry},
	node::BlockNumber,
	schema::{SchemaId, SchemaValidator},
	utils::wrap_binary_data,
};
use common_runtime::extensions::check_nonce::CheckNonce;

#[test]
fn it_creates_an_msa_account() {
	new_test_ext().execute_with(|| {
		assert_ok!(Msa::create(test_origin_signed(1)));

		assert_eq!(Msa::get_msa_by_public_key(test_public(1)), Some(1 as MessageSourceId));

		assert_eq!(Msa::get_current_msa_identifier_maximum(), 1);

		System::assert_last_event(Event::MsaCreated { msa_id: 1, key: test_public(1) }.into());
	});
}

#[test]
fn it_throws_msa_identifier_overflow() {
	new_test_ext().execute_with(|| {
		CurrentMsaIdentifierMaximum::<Test>::set(u64::MAX);

		assert_noop!(Msa::create(test_origin_signed(1)), Error::<Test>::MsaIdOverflow);
	});
}

#[test]
#[allow(unused_must_use)]
fn it_does_not_allow_duplicate_keys() {
	new_test_ext().execute_with(|| {
		Msa::create(test_origin_signed(1));

		assert_noop!(Msa::create(test_origin_signed(1)), Error::<Test>::KeyAlreadyRegistered);

		assert_eq!(Msa::get_current_msa_identifier_maximum(), 1);
	});
}

#[test]
fn it_create_has_weight() {
	new_test_ext().execute_with(|| {
		let call = MsaCall::<Test>::create {};
		let dispatch_info = call.get_dispatch_info();

		assert!(dispatch_info.weight > Weight::from_ref_time(10_000 as u64));
	});
}

#[test]
fn it_throws_error_when_key_verification_fails() {
	new_test_ext().execute_with(|| {
		let (new_msa_id, owner_key_pair) = create_account();
		let (new_key_pair, _) = sr25519::Pair::generate();
		let (fake_key_pair, _) = sr25519::Pair::generate();

		let add_new_key_data = AddKeyData { msa_id: new_msa_id, expiration: 10 };
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let owner_signature: MultiSignature = owner_key_pair.sign(&encode_data_new_key_data).into();
		let fake_signature: MultiSignature = fake_key_pair.sign(&encode_data_new_key_data).into();

		assert_noop!(
			Msa::add_public_key_to_msa(
				test_origin_signed(1),
				owner_key_pair.public().into(),
				owner_signature,
				new_key_pair.public().into(), // This is different from the signature signer below
				fake_signature,
				add_new_key_data
			),
			Error::<Test>::AddKeySignatureVerificationFailed
		);
	});
}

#[test]
fn it_throws_error_when_not_msa_owner() {
	new_test_ext().execute_with(|| {
		let (new_msa_id, _) = create_account();
		let (new_key_pair, _) = sr25519::Pair::generate();
		let (_fake_msa_id, fake_key_pair) = create_account();

		assert_ok!(Msa::create_account(test_public(1), EMPTY_FUNCTION));

		let add_new_key_data = AddKeyData { msa_id: new_msa_id, expiration: 10 };
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let fake_owner_signature: MultiSignature =
			fake_key_pair.sign(&encode_data_new_key_data).into();
		let new_key_signature: MultiSignature = new_key_pair.sign(&encode_data_new_key_data).into();

		assert_noop!(
			Msa::add_public_key_to_msa(
				test_origin_signed(1),
				fake_key_pair.public().into(),
				fake_owner_signature,
				new_key_pair.public().into(),
				new_key_signature,
				add_new_key_data
			),
			Error::<Test>::NotMsaOwner
		);
	});
}

#[test]
fn it_throws_error_when_for_duplicate_key() {
	new_test_ext().execute_with(|| {
		let (new_msa_id, owner_key_pair) = create_account();
		let (new_key_pair, _) = sr25519::Pair::generate();

		let _ = Msa::add_key(new_msa_id, &new_key_pair.public().into(), EMPTY_FUNCTION);

		let add_new_key_data = AddKeyData { msa_id: new_msa_id, expiration: 10 };
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let owner_signature: MultiSignature = owner_key_pair.sign(&encode_data_new_key_data).into();
		let new_key_signature: MultiSignature = new_key_pair.sign(&encode_data_new_key_data).into();

		assert_noop!(
			Msa::add_public_key_to_msa(
				test_origin_signed(1),
				owner_key_pair.public().into(),
				owner_signature,
				new_key_pair.public().into(),
				new_key_signature,
				add_new_key_data
			),
			Error::<Test>::KeyAlreadyRegistered
		);
	});
}

#[test]
fn add_key_with_more_than_allowed_should_panic() {
	new_test_ext().execute_with(|| {
		// arrange
		let (new_msa_id, owner_key_pair) = create_account();

		for _ in 1..<Test as Config>::MaxPublicKeysPerMsa::get() {
			let (new_key_pair, _) = sr25519::Pair::generate();

			let add_new_key_data = AddKeyData { msa_id: new_msa_id, expiration: 10 };
			let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

			let owner_signature: MultiSignature =
				owner_key_pair.sign(&encode_data_new_key_data).into();
			let new_key_signature: MultiSignature =
				new_key_pair.sign(&encode_data_new_key_data).into();

			assert_ok!(Msa::add_public_key_to_msa(
				test_origin_signed(1),
				owner_key_pair.public().into(),
				owner_signature,
				new_key_pair.public().into(),
				new_key_signature,
				add_new_key_data
			));
		}

		// act
		let (final_key_pair, _) = sr25519::Pair::generate();
		let add_new_key_data = AddKeyData { msa_id: new_msa_id, expiration: 10 };
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let owner_signature: MultiSignature = owner_key_pair.sign(&encode_data_new_key_data).into();
		let final_key_signature: MultiSignature =
			final_key_pair.sign(&encode_data_new_key_data).into();
		assert_noop!(
			Msa::add_public_key_to_msa(
				test_origin_signed(1),
				owner_key_pair.public().into(),
				owner_signature,
				final_key_pair.public().into(),
				final_key_signature,
				add_new_key_data
			),
			Error::<Test>::KeyLimitExceeded
		);
	});
}

#[test]
fn add_key_with_valid_request_should_store_value_and_event() {
	new_test_ext().execute_with(|| {
		// arrange
		let (new_msa_id, owner_key_pair) = create_account();
		let (new_key_pair, _) = sr25519::Pair::generate();

		let add_new_key_data = AddKeyData { msa_id: new_msa_id, expiration: 10 };
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let owner_signature: MultiSignature = owner_key_pair.sign(&encode_data_new_key_data).into();
		let new_key_signature: MultiSignature = new_key_pair.sign(&encode_data_new_key_data).into();

		// act
		assert_ok!(Msa::add_public_key_to_msa(
			test_origin_signed(1),
			owner_key_pair.public().into(),
			owner_signature,
			new_key_pair.public().into(),
			new_key_signature,
			add_new_key_data
		));

		// assert
		// *Temporarily Removed* until https://github.com/LibertyDSNP/frequency/issues/418// *Temporarily Removed* until https://github.com/LibertyDSNP/frequency/issues/418
		// let keys = Msa::fetch_msa_keys(new_msa_id);
		// assert_eq!(keys.len(), 2);
		// assert_eq!{keys.contains(&KeyInfoResponse {key: AccountId32::from(new_key), msa_id: new_msa_id}), true}

		let keys_count = Msa::get_public_key_count_by_msa_id(new_msa_id);
		assert_eq!(keys_count, 2);
		System::assert_last_event(
			Event::PublicKeyAdded { msa_id: 1, key: new_key_pair.public().into() }.into(),
		);
	});
}

/// Assert that when attempting to add a key to an MSA with an expired proof that the key is NOT added.
/// Expected error: ProofHasExpired
#[test]
fn add_key_with_expired_proof_fails() {
	new_test_ext().execute_with(|| {
		// arrange
		let (new_msa_id, owner_key_pair) = create_account();
		let (new_key_pair, _) = sr25519::Pair::generate();

		// The current block is 1, therefore setting the proof expiration to 1 should cause
		// the extrinsic to fail because the proof has expired.
		let add_new_key_data = AddKeyData { msa_id: new_msa_id, expiration: 1 };
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let owner_signature: MultiSignature = owner_key_pair.sign(&encode_data_new_key_data).into();
		let new_key_signature: MultiSignature = new_key_pair.sign(&encode_data_new_key_data).into();

		assert_noop!(
			Msa::add_public_key_to_msa(
				test_origin_signed(1),
				owner_key_pair.public().into(),
				owner_signature,
				new_key_pair.public().into(),
				new_key_signature,
				add_new_key_data
			),
			Error::<Test>::ProofHasExpired
		);
	})
}

/// Assert that when attempting to add a key to an MSA with a proof expiration too far into the future the key is NOT added.
/// Expected error: ProofNotYetValid
#[test]
fn add_key_with_proof_too_far_into_future_fails() {
	new_test_ext().execute_with(|| {
		// arrange
		let (new_msa_id, owner_key_pair) = create_account();
		let (new_key_pair, _) = sr25519::Pair::generate();

		// The current block is 1, therefore setting the proof expiration to EXPIRATION_BLOCK_VALIDITY_GAP + 1
		// should cause the extrinsic to fail because the proof is only valid for EXPIRATION_BLOCK_VALIDITY_GAP
		// more blocks.
		let add_new_key_data = AddKeyData { msa_id: new_msa_id, expiration: 202 };
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let owner_signature: MultiSignature = owner_key_pair.sign(&encode_data_new_key_data).into();
		let new_key_signature: MultiSignature = new_key_pair.sign(&encode_data_new_key_data).into();

		assert_noop!(
			Msa::add_public_key_to_msa(
				test_origin_signed(1),
				owner_key_pair.public().into(),
				owner_signature,
				new_key_pair.public().into(),
				new_key_signature,
				add_new_key_data
			),
			Error::<Test>::ProofNotYetValid
		);
	})
}

#[test]
fn it_deletes_msa_key_successfully() {
	new_test_ext().execute_with(|| {
		assert_ok!(Msa::add_key(2, &test_public(1), EMPTY_FUNCTION));
		assert_ok!(Msa::add_key(2, &test_public(2), EMPTY_FUNCTION));

		assert_ok!(Msa::delete_msa_public_key(test_origin_signed(1), test_public(2)));

		let info = Msa::get_msa_by_public_key(&test_public(2));

		assert_eq!(info, None);

		System::assert_last_event(Event::PublicKeyDeleted { key: test_public(2) }.into());
	})
}

#[test]
fn it_deletes_msa_last_key_self_removal() {
	new_test_ext().execute_with(|| {
		let msa_id = 2;

		// Create an account
		let test_account = test_public(4);
		let origin = Origin::signed(test_account.clone());

		// Add an account to the MSA so it has exactly one account
		assert_ok!(Msa::add_key(msa_id, &test_account, EMPTY_FUNCTION));

		// Attempt to delete/remove the account from the MSA
		assert_noop!(
			Msa::delete_msa_public_key(origin, test_account),
			Error::<Test>::InvalidSelfRemoval
		);
	})
}

#[test]
fn test_retire_msa_success() {
	new_test_ext().execute_with(|| {
		let (test_account_key_pair, _) = sr25519::Pair::generate();

		// Create an account
		let test_account = AccountId32::new(test_account_key_pair.public().into());
		let origin = Origin::signed(test_account.clone());

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

		let add_new_key_data = AddKeyData { msa_id: msa_id2, expiration: 10 };
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());
		let signature: MultiSignature = key_pair1.sign(&encode_data_new_key_data).into();
		assert_noop!(
			Msa::add_public_key_to_msa(
				Origin::signed(test_account.clone()),
				new_account1.into(),
				signature.clone(),
				new_account1.into(),
				signature,
				add_new_key_data
			),
			Error::<Test>::NoKeyExists
		);

		// [TEST] Adding a provider to the retired MSA should fail
		let (provider_key_pair, _) = sr25519::Pair::generate();
		let provider_account = provider_key_pair.public();

		// Create provider account and get its MSA ID (u64)
		assert_ok!(Msa::create(Origin::signed(provider_account.into())));
		let provider_msa_id =
			Msa::ensure_valid_msa_key(&AccountId32::new(provider_account.0)).unwrap();

		assert_ok!(Msa::create_provider(Origin::signed(provider_account.into()), Vec::from("Foo")));

		let (delegator_signature, add_provider_payload) =
			create_and_sign_add_provider_payload(test_account_key_pair, provider_msa_id);

		assert_noop!(
			Msa::grant_delegation(
				Origin::signed(provider_account.into()),
				test_account.clone(),
				delegator_signature,
				add_provider_payload
			),
			Error::<Test>::NoKeyExists
		);

		// [TEST] Revoking a provider (modifying permissions) should fail
		assert_noop!(
			Msa::revoke_delegation_by_delegator(
				Origin::signed(test_account.clone()),
				provider_msa_id
			),
			Error::<Test>::NoKeyExists
		);
	})
}

#[test]
fn test_retire_msa_does_nothing_when_no_msa() {
	new_test_ext().execute_with(|| {
		let (test_pair, _) = sr25519::Pair::generate();
		let first_account_key = test_pair.public();
		let origin = Origin::signed(first_account_key.into());

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
		let origin = Origin::signed(test_account.clone());

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
pub fn test_get_owner_of() {
	new_test_ext().execute_with(|| {
		assert_eq!(Msa::get_owner_of(&test_public(1)), None);

		assert_ok!(Msa::create(test_origin_signed(1)));

		assert_eq!(Msa::get_owner_of(&test_public(1)), Some(1));
	});
}

#[test]
pub fn test_delete_key() {
	new_test_ext().execute_with(|| {
		assert_ok!(Msa::add_key(1, &test_public(1), EMPTY_FUNCTION));

		let info = Msa::get_msa_by_public_key(&test_public(1));

		assert_eq!(info, Some(1 as MessageSourceId));

		assert_ok!(Msa::delete_key_for_msa(info.unwrap(), &test_public(1)));
	});
}

#[test]
pub fn test_delete_key_errors() {
	new_test_ext().execute_with(|| {
		assert_ok!(Msa::add_key(1, &test_public(1), EMPTY_FUNCTION));

		assert_ok!(Msa::delete_key_for_msa(1, &test_public(1)));
	});
}

#[test]
pub fn test_ensure_msa_owner() {
	new_test_ext().execute_with(|| {
		assert_noop!(Msa::ensure_msa_owner(&test_public(1), 1), Error::<Test>::NoKeyExists);

		assert_ok!(Msa::add_key(1, &test_public(1), EMPTY_FUNCTION));

		assert_eq!(Msa::ensure_msa_owner(&test_public(1), 1), Ok(()));
	});
}

#[test]
pub fn add_provider_to_msa_is_success() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let (delegator_pair, _) = sr25519::Pair::generate();
		let delegator_account = delegator_pair.public();

		// Create provider account and get its MSA ID (u64)
		assert_ok!(Msa::create(Origin::signed(provider_account.into())));
		let provider_msa =
			Msa::ensure_valid_msa_key(&AccountId32::new(provider_account.0)).unwrap();

		// Create delegator account and get its MSA ID (u64)
		assert_ok!(Msa::create(Origin::signed(delegator_account.into())));
		let delegator_msa =
			Msa::ensure_valid_msa_key(&AccountId32::new(delegator_account.0)).unwrap();

		// Register provider
		assert_ok!(Msa::create_provider(Origin::signed(provider_account.into()), Vec::from("Foo")));

		let (delegator_signature, add_provider_payload) =
			create_and_sign_add_provider_payload(delegator_pair, provider_msa);

		set_schema_count::<Test>(10);

		assert_ok!(Msa::grant_delegation(
			Origin::signed(provider_account.into()),
			delegator_account.into(),
			delegator_signature,
			add_provider_payload
		));

		let provider = Provider(provider_msa);
		let delegator = Delegator(delegator_msa);

		assert_eq!(
			Msa::get_delegation(delegator, provider),
			Some(Delegation { revoked_at: 0, schema_permissions: Default::default() })
		);

		System::assert_last_event(
			Event::DelegationGranted {
				delegator: delegator_msa.into(),
				provider: provider_msa.into(),
			}
			.into(),
		);
	});
}

#[test]
pub fn grant_delegation_to_msa_throws_add_provider_verification_failed() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let account = key_pair.public();
		let expiration: BlockNumber = 10;
		let add_provider_payload = AddProvider::new(2, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = key_pair.sign(&encode_add_provider_data).into();
		let fake_provider_payload = AddProvider::new(3, None, expiration);
		assert_noop!(
			Msa::grant_delegation(
				Origin::signed(account.into()),
				account.into(),
				signature,
				fake_provider_payload
			),
			Error::<Test>::AddProviderSignatureVerificationFailed
		);
	});
}

#[test]
pub fn grant_delegation_throws_no_key_exist_error() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let expiration: BlockNumber = 10;
		let add_provider_payload = AddProvider::new(2, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = key_pair.sign(&encode_add_provider_data).into();

		assert_noop!(
			Msa::grant_delegation(
				test_origin_signed(1),
				provider_account.into(),
				signature,
				add_provider_payload
			),
			Error::<Test>::NoKeyExists
		);
	});
}

#[test]
pub fn grant_delegation_throws_key_revoked_error() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let expiration: BlockNumber = 10;
		let add_provider_payload = AddProvider::new(2, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = key_pair.sign(&encode_add_provider_data).into();

		assert_ok!(Msa::create(test_origin_signed(1)));
		assert_ok!(Msa::create(Origin::signed(provider_account.into())));
		assert_ok!(Msa::delete_key_for_msa(1, &test_public(1)));

		assert_noop!(
			Msa::grant_delegation(
				test_origin_signed(1),
				provider_account.into(),
				signature,
				add_provider_payload
			),
			Error::<Test>::NoKeyExists
		);
	});
}

#[test]
pub fn grant_delegation_throws_invalid_self_provider_error() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let expiration: BlockNumber = 10;
		let add_provider_payload = AddProvider::new(1, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());
		let signature: MultiSignature = key_pair.sign(&encode_add_provider_data).into();

		assert_ok!(Msa::create(Origin::signed(provider_account.into())));

		assert_noop!(
			Msa::grant_delegation(
				Origin::signed(provider_account.into()),
				provider_account.into(),
				signature,
				add_provider_payload
			),
			Error::<Test>::InvalidSelfProvider
		);
	});
}

#[test]
pub fn grant_delegation_throws_unauthorized_delegator_error() {
	new_test_ext().execute_with(|| {
		// Generate a key pair for the provider
		let (provider_key_pair, _) = sr25519::Pair::generate();
		let provider_account = provider_key_pair.public();

		// Generate a key pair for the delegator
		let (delegator_key_pair, _) = sr25519::Pair::generate();
		let delegator_account = delegator_key_pair.public();
		assert_ok!(Msa::create(Origin::signed(delegator_account.into())));
		let delegator_msa_id =
			Msa::ensure_valid_msa_key(&AccountId32::new(delegator_account.0)).unwrap();

		let expiration: BlockNumber = 10;
		let add_provider_payload = AddProvider::new(delegator_msa_id, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());
		let signature: MultiSignature = delegator_key_pair.sign(&encode_add_provider_data).into();

		assert_ok!(Msa::create(Origin::signed(provider_account.into())));

		// Register provider
		assert_ok!(Msa::create_provider(Origin::signed(provider_account.into()), Vec::from("Foo")));

		assert_noop!(
			Msa::grant_delegation(
				Origin::signed(provider_account.into()),
				delegator_account.into(),
				signature,
				add_provider_payload
			),
			Error::<Test>::UnauthorizedDelegator
		);
	});
}

#[test]
pub fn ensure_valid_msa_key_is_successfull() {
	new_test_ext().execute_with(|| {
		assert_noop!(Msa::ensure_valid_msa_key(&test_public(1)), Error::<Test>::NoKeyExists);

		assert_ok!(Msa::create(test_origin_signed(1)));

		assert_ok!(Msa::ensure_valid_msa_key(&test_public(1)));
	});
}

#[test]
pub fn create_sponsored_account_with_delegation_with_valid_input_should_succeed() {
	new_test_ext().execute_with(|| {
		// arrange
		let (provider_msa, provider_key_pair) = create_account();
		let provider_account = provider_key_pair.public();
		// Register provider
		assert_ok!(Msa::create_provider(Origin::signed(provider_account.into()), Vec::from("Foo")));

		let (key_pair_delegator, _) = sr25519::Pair::generate();
		let delegator_account = key_pair_delegator.public();

		let expiration: BlockNumber = 10;

		let add_provider_payload = AddProvider::new(provider_msa, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = key_pair_delegator.sign(&encode_add_provider_data).into();

		// act
		assert_ok!(Msa::create_sponsored_account_with_delegation(
			Origin::signed(provider_account.into()),
			delegator_account.into(),
			signature,
			add_provider_payload
		));

		// assert
		let delegator_msa =
			Msa::get_msa_by_public_key(&AccountId32::new(delegator_account.0)).unwrap();

		let provider_info = Msa::get_delegation(Delegator(2), Provider(1));
		assert_eq!(provider_info.is_some(), true);

		let events_occured = System::events();
		let created_event = &events_occured.as_slice()[1];
		let provider_event = &events_occured.as_slice()[2];
		assert_eq!(
			created_event.event,
			Event::MsaCreated { msa_id: delegator_msa, key: delegator_account.into() }.into()
		);
		assert_eq!(
			provider_event.event,
			Event::DelegationGranted {
				provider: provider_msa.into(),
				delegator: delegator_msa.into()
			}
			.into()
		);
	});
}

#[test]
fn create_sponsored_account_with_delegation_with_invalid_signature_should_fail() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let (key_pair_delegator, _) = sr25519::Pair::generate();
		let delegator_account = key_pair_delegator.public();

		let (signer_pair, _) = sr25519::Pair::generate();

		let expiration: BlockNumber = 10;
		let add_provider_payload = AddProvider::new(1u64, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = signer_pair.sign(&encode_add_provider_data).into();

		assert_ok!(Msa::create(Origin::signed(provider_account.into())));

		// act
		assert_noop!(
			Msa::create_sponsored_account_with_delegation(
				Origin::signed(provider_account.into()),
				delegator_account.into(),
				signature,
				add_provider_payload
			),
			Error::<Test>::InvalidSignature
		);
	});
}

#[test]
pub fn create_sponsored_account_with_delegation_with_invalid_add_provider_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let (key_pair_delegator, _) = sr25519::Pair::generate();
		let delegator_account = key_pair_delegator.public();

		let expiration: BlockNumber = 10;
		let add_provider_payload = AddProvider::new(1u64, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = key_pair_delegator.sign(&encode_add_provider_data).into();

		assert_ok!(Msa::create(Origin::signed(provider_account.into())));
		assert_ok!(Msa::create(Origin::signed(delegator_account.into())));

		// Register provider
		assert_ok!(Msa::create_provider(Origin::signed(provider_account.into()), Vec::from("Foo")));

		// act
		assert_noop!(
			Msa::create_sponsored_account_with_delegation(
				Origin::signed(provider_account.into()),
				delegator_account.into(),
				signature,
				add_provider_payload
			),
			Error::<Test>::KeyAlreadyRegistered
		);
	});
}

#[test]
pub fn create_sponsored_account_with_delegation_with_different_authorized_msa_id_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let (key_pair_delegator, _) = sr25519::Pair::generate();
		let delegator_account = key_pair_delegator.public();

		let expiration: BlockNumber = 10;
		let add_provider_payload = AddProvider::new(3u64, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = key_pair_delegator.sign(&encode_add_provider_data).into();

		assert_ok!(Msa::create(Origin::signed(provider_account.into())));

		// act
		assert_noop!(
			Msa::create_sponsored_account_with_delegation(
				Origin::signed(provider_account.into()),
				delegator_account.into(),
				signature,
				add_provider_payload
			),
			Error::<Test>::UnauthorizedProvider
		);
	});
}

#[test]
pub fn create_sponsored_account_with_delegation_expired() {
	new_test_ext().execute_with(|| {
		// arrange
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let (key_pair_delegator, _) = sr25519::Pair::generate();
		let delegator_account = key_pair_delegator.public();

		let expiration: BlockNumber = 0;

		let add_provider_payload = AddProvider::new(1u64, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = key_pair_delegator.sign(&encode_add_provider_data).into();

		assert_ok!(Msa::create(Origin::signed(provider_account.into())));

		// Register provider
		assert_ok!(Msa::create_provider(Origin::signed(provider_account.into()), Vec::from("Foo")));

		// act
		assert_noop!(
			Msa::create_sponsored_account_with_delegation(
				Origin::signed(provider_account.into()),
				delegator_account.into(),
				signature,
				add_provider_payload
			),
			Error::<Test>::ProofHasExpired
		);
	});
}

#[test]
pub fn add_key_with_panic_in_on_success_should_revert_everything() {
	new_test_ext().execute_with(|| {
		// arrange
		let msa_id = 1u64;
		let key = test_public(msa_id as u8);

		// act
		assert_noop!(
			Msa::add_key(msa_id, &key, |new_msa_id| -> DispatchResult {
				ensure!(new_msa_id != msa_id, Error::<Test>::InvalidSelfRemoval);
				Ok(())
			}),
			Error::<Test>::InvalidSelfRemoval
		);

		// assert
		assert_eq!(Msa::get_msa_by_public_key(&key), None);

		// *Temporarily Removed* until https://github.com/LibertyDSNP/frequency/issues/418 is completed
		// assert_eq!(Msa::get_msa_keys(msa_id).into_inner(), vec![])
	});
}

#[test]
pub fn create_account_with_panic_in_on_success_should_revert_everything() {
	new_test_ext().execute_with(|| {
		// arrange
		let msa_id = 1u64;
		let key = test_public(msa_id as u8);
		let next_msa_id = Msa::get_next_msa_id().unwrap();

		// act
		assert_noop!(
			Msa::create_account(key, |new_msa_id| -> DispatchResult {
				ensure!(new_msa_id != msa_id, Error::<Test>::InvalidSelfRemoval);
				Ok(())
			}),
			Error::<Test>::InvalidSelfRemoval
		);

		// assert
		assert_eq!(next_msa_id, Msa::get_next_msa_id().unwrap());
	});
}

#[test]
pub fn revoke_delegation_by_delegator_is_successful() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let (delegator_pair, _) = sr25519::Pair::generate();
		let delegator_account = delegator_pair.public();

		assert_ok!(Msa::create(Origin::signed(delegator_account.into())));
		assert_ok!(Msa::create(Origin::signed(provider_account.into())));

		// Register provider
		assert_ok!(Msa::create_provider(Origin::signed(provider_account.into()), Vec::from("Foo")));

		let provider_msa =
			Msa::ensure_valid_msa_key(&AccountId32::new(provider_account.0)).unwrap();

		let (delegator_signature, add_provider_payload) =
			create_and_sign_add_provider_payload(delegator_pair, provider_msa);

		assert_ok!(Msa::grant_delegation(
			Origin::signed(provider_account.into()),
			delegator_account.into(),
			delegator_signature,
			add_provider_payload
		));

		assert_ok!(Msa::revoke_delegation_by_delegator(
			Origin::signed(delegator_account.into()),
			2
		));

		System::assert_last_event(
			Event::DelegationRevoked { delegator: 1.into(), provider: 2.into() }.into(),
		);
	});
}

#[test]
pub fn revoke_provider_is_successful() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let (delegator_pair, _) = sr25519::Pair::generate();
		let delegator_account = delegator_pair.public();

		assert_ok!(Msa::create(Origin::signed(delegator_account.into())));
		assert_ok!(Msa::create(Origin::signed(provider_account.into())));

		let provider_msa =
			Msa::ensure_valid_msa_key(&AccountId32::new(provider_account.0)).unwrap();

		let (delegator_signature, add_provider_payload) =
			create_and_sign_add_provider_payload(delegator_pair, provider_msa);

		// Register provider
		assert_ok!(Msa::create_provider(Origin::signed(provider_account.into()), Vec::from("Foo")));

		assert_ok!(Msa::grant_delegation(
			Origin::signed(provider_account.into()),
			delegator_account.into(),
			delegator_signature,
			add_provider_payload
		));

		let delegator_msa =
			Msa::ensure_valid_msa_key(&AccountId32::new(delegator_account.0)).unwrap();

		let provider = Provider(provider_msa);
		let delegator = Delegator(delegator_msa);

		assert_ok!(Msa::revoke_provider(provider, delegator));

		assert_eq!(
			Msa::get_delegation(delegator, provider).unwrap(),
			Delegation { revoked_at: 1, schema_permissions: Default::default() },
		);
	});
}

#[test]
fn revoke_delegation_by_delegator_fails_when_no_msa() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Msa::revoke_delegation_by_delegator(test_origin_signed(1), 1),
			Error::<Test>::NoKeyExists
		);
	});
}

#[test]
pub fn revoke_delegation_fails_if_only_key_is_revoked() {
	new_test_ext().execute_with(|| {
		assert_ok!(Msa::create(test_origin_signed(2)));
		assert_ok!(Msa::delete_key_for_msa(1, &test_public(2)));
		assert_noop!(
			Msa::revoke_delegation_by_delegator(test_origin_signed(2), 1),
			Error::<Test>::NoKeyExists
		);
	})
}

#[test]
pub fn revoke_delegation_by_delegator_fails_if_has_msa_but_no_delegation() {
	new_test_ext().execute_with(|| {
		assert_ok!(Msa::create(test_origin_signed(1)));
		assert_ok!(Msa::create(test_origin_signed(2)));
		assert_noop!(
			Msa::revoke_delegation_by_delegator(test_origin_signed(1), 2),
			Error::<Test>::DelegationNotFound
		);
	})
}

#[test]
fn revoke_delegation_by_delegator_throws_error_when_delegation_already_revoked() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let (delegator_pair, _) = sr25519::Pair::generate();
		let delegator_account = delegator_pair.public();

		assert_ok!(Msa::create(Origin::signed(delegator_account.into())));
		assert_ok!(Msa::create(Origin::signed(provider_account.into())));

		let provider_msa =
			Msa::ensure_valid_msa_key(&AccountId32::new(provider_account.0)).unwrap();

		let (delegator_signature, add_provider_payload) =
			create_and_sign_add_provider_payload(delegator_pair, provider_msa);

		// Register provider
		assert_ok!(Msa::create_provider(Origin::signed(provider_account.into()), Vec::from("Foo")));

		assert_ok!(Msa::grant_delegation(
			Origin::signed(provider_account.into()),
			delegator_account.into(),
			delegator_signature,
			add_provider_payload
		));

		assert_ok!(Msa::revoke_delegation_by_delegator(
			Origin::signed(delegator_account.into()),
			provider_msa
		));

		assert_noop!(
			Msa::revoke_delegation_by_delegator(
				Origin::signed(delegator_account.into()),
				provider_msa
			),
			Error::<Test>::DelegationRevoked
		);
	});
}

/// Assert that the call to revoke a delegation is free.
#[test]
pub fn revoke_provider_call_has_no_cost() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let expiration: BlockNumber = 10;
		let add_provider_payload = AddProvider::new(1, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = key_pair.sign(&encode_add_provider_data).into();

		assert_ok!(Msa::create(test_origin_signed(1)));
		assert_ok!(Msa::create(Origin::signed(provider_account.into())));

		// Register provider
		assert_ok!(Msa::create_provider(test_origin_signed(1), Vec::from("Foo")));

		assert_ok!(Msa::grant_delegation(
			test_origin_signed(1),
			provider_account.into(),
			signature,
			add_provider_payload
		));

		let call = MsaCall::<Test>::revoke_delegation_by_delegator { provider_msa_id: 2 };
		let dispatch_info = call.get_dispatch_info();

		assert_eq!(dispatch_info.pays_fee, Pays::No);
	})
}

#[test]
pub fn revoke_delegation_by_provider_happy_path() {
	new_test_ext().execute_with(|| {
		// 1. create two key pairs
		let (provider_pair, _) = sr25519::Pair::generate();
		let (user_pair, _) = sr25519::Pair::generate();

		let provider_key = provider_pair.public();
		let delegator_key = user_pair.public();

		// 2. create provider MSA
		assert_ok!(Msa::create(Origin::signed(provider_key.into()))); // MSA = 1

		// Register provider
		assert_ok!(Msa::create_provider(Origin::signed(provider_key.into()), Vec::from("Foo")));

		// 3. create delegator MSA and provider to provider
		let expiration: BlockNumber = 10;
		let add_provider_payload = AddProvider::new(1u64, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());
		let signature: MultiSignature = user_pair.sign(&encode_add_provider_data).into();
		// 3.5 create the user's MSA + add provider as provider
		assert_ok!(Msa::create_sponsored_account_with_delegation(
			Origin::signed(provider_key.into()),
			delegator_key.into(),
			signature,
			add_provider_payload
		));

		//  4. set some block number to ensure it's not a default value
		System::set_block_number(System::block_number() + 25);

		// 5. assert_ok! fn as 2 to remove provider 1
		assert_ok!(Msa::revoke_delegation_by_provider(Origin::signed(provider_key.into()), 2u64));

		// 6. verify that the provider is revoked
		let provider_info = Msa::get_delegation(Delegator(2), Provider(1));
		assert_eq!(
			provider_info,
			Some(Delegation { revoked_at: 26, schema_permissions: Default::default() })
		);

		// 7. verify the event
		System::assert_last_event(
			Event::DelegationRevoked { provider: Provider(1), delegator: Delegator(2) }.into(),
		);
	})
}

#[test]
pub fn revoke_delegation_by_provider_has_correct_costs() {
	new_test_ext().execute_with(|| {
		let call = MsaCall::<Test>::revoke_delegation_by_provider { delegator: 2 };
		let dispatch_info = call.get_dispatch_info();

		assert_eq!(dispatch_info.pays_fee, Pays::No);
	})
}

#[test]
pub fn revoke_delegation_by_provider_does_nothing_when_no_msa() {
	new_test_ext().execute_with(|| {
		let (provider_pair, _) = sr25519::Pair::generate();

		let provider_key = provider_pair.public();
		let delegator_msa = 2u64;

		let current_event_count = System::event_count();
		assert_ok!(Msa::revoke_delegation_by_provider(
			Origin::signed(provider_key.into()),
			delegator_msa
		));
		// no events should have been deposited.
		assert_eq!(current_event_count, System::event_count());
	});
}

#[test]
pub fn valid_delegation() {
	new_test_ext().execute_with(|| {
		let provider = Provider(1);
		let delegator = Delegator(2);

		assert_ok!(Msa::add_provider(provider, delegator, Vec::default()));

		System::set_block_number(System::block_number() + 1);

		assert_ok!(Msa::ensure_valid_delegation(provider, delegator, None));
	})
}

#[test]
pub fn delegation_not_found() {
	new_test_ext().execute_with(|| {
		let provider = Provider(1);
		let delegator = Delegator(2);

		assert_noop!(
			Msa::ensure_valid_delegation(provider, delegator, None),
			Error::<Test>::DelegationNotFound
		);
	})
}

#[test]
pub fn delegation_expired() {
	new_test_ext().execute_with(|| {
		let provider = Provider(1);
		let delegator = Delegator(2);

		assert_ok!(Msa::add_provider(provider, delegator, Vec::default()));

		System::set_block_number(System::block_number() + 1);
		assert_ok!(Msa::ensure_valid_delegation(provider, delegator, None));

		assert_ok!(Msa::revoke_provider(provider, delegator));

		System::set_block_number(System::block_number() + 1);

		assert_noop!(
			Msa::ensure_valid_delegation(provider, delegator, None),
			Error::<Test>::DelegationRevoked
		);
	})
}

/// Assert that revoking an MSA delegation passes the signed extension CheckFreeExtrinsicUse
/// validation when a valid delegation exists.
#[test]
fn signed_extension_revoke_delegation_by_delegator_success() {
	new_test_ext().execute_with(|| {
		let (provider_msa_id, delegator_account) = create_provider_msa_and_delegator();
		let call_revoke_delegation: &<Test as frame_system::Config>::Call =
			&Call::Msa(MsaCall::revoke_delegation_by_delegator { provider_msa_id });
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
		let call_revoke_delegation: &<Test as frame_system::Config>::Call =
			&Call::Msa(MsaCall::revoke_delegation_by_delegator { provider_msa_id });
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
			Origin::signed(delegator_account.into()),
			provider_msa_id
		));

		System::set_block_number(System::block_number() + 1);
		let call_revoke_delegation: &<Test as frame_system::Config>::Call =
			&Call::Msa(MsaCall::revoke_delegation_by_delegator { provider_msa_id });
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
		let call_revoke_delegation: &<Test as frame_system::Config>::Call =
			&Call::Msa(MsaCall::revoke_delegation_by_provider { delegator: delegator_msa_id });
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

fn assert_revoke_delegation_by_provider_err(
	expected_err: InvalidTransaction,
	provider_account: Public,
	delegator_msa_id: u64,
) {
	let call_revoke_delegation: &<Test as frame_system::Config>::Call =
		&Call::Msa(MsaCall::revoke_delegation_by_provider { delegator: delegator_msa_id });
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

		let (delegator_msa, delegator_pair) = create_account();

		let expected_err = InvalidTransaction::Custom(ValidityError::InvalidMsaKey as u8);
		assert_revoke_delegation_by_provider_err(expected_err, provider_account, delegator_msa);
	});
}

#[test]
fn signed_extension_revoke_delegation_by_provider_fails_when_no_delegation() {
	new_test_ext().execute_with(|| {
		let (_, provider_pair) = create_account();
		let provider_account = provider_pair.public();
		let (delegator_msa, delegator_pair) = create_account();
		let delegator_account = delegator_pair.public();

		let expected_err = InvalidTransaction::Custom(ValidityError::InvalidDelegation as u8);
		assert_revoke_delegation_by_provider_err(expected_err, provider_account, delegator_msa);
	})
}

/// Assert that a call that is not revoke_delegation_by_delegator passes the signed extension
/// CheckFreeExtrinsicUse validation.
#[test]
fn signed_extension_validation_valid_for_other_extrinsics() {
	let random_call_should_pass: &<Test as frame_system::Config>::Call =
		&Call::Msa(MsaCall::create {});
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

#[test]
pub fn delete_msa_public_key_call_has_correct_costs() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let new_key = key_pair.public();

		let call = MsaCall::<Test>::delete_msa_public_key { key: AccountId32::from(new_key) };
		let dispatch_info = call.get_dispatch_info();
		assert_eq!(dispatch_info.pays_fee, Pays::No);
	})
}

#[test]
fn signed_extension_validation_delete_msa_public_key_success() {
	new_test_ext().execute_with(|| {
		let (owner_msa_id, owner_key_pair) = create_account();

		let (user_key_pair, _) = sr25519::Pair::generate();
		let user_public_key = user_key_pair.public();
		let user_account_id = AccountId32::from(user_public_key);
		assert_ok!(Msa::add_key(owner_msa_id, &user_account_id, EMPTY_FUNCTION));

		let call_delete_msa_public_key: &<Test as frame_system::Config>::Call =
			&Call::Msa(MsaCall::delete_msa_public_key { key: owner_key_pair.public().into() });

		let info = DispatchInfo::default();
		let len = 0_usize;
		let result = CheckFreeExtrinsicUse::<Test>::new().validate(
			&owner_key_pair.public().into(),
			call_delete_msa_public_key,
			&info,
			len,
		);
		assert_ok!(result);
		assert_ok!(Msa::delete_msa_public_key(
			Origin::signed(AccountId32::from(owner_key_pair.public())),
			user_account_id
		));
	});
}

#[test]
fn signed_extension_validation_failure_when_delete_msa_public_key_called_twice() {
	new_test_ext().execute_with(|| {
		let (owner_msa_id, owner_key_pair) = create_account();

		let (user_key_pair, _) = sr25519::Pair::generate();
		let user_public_key = user_key_pair.public();
		let user_account_id = AccountId32::from(user_public_key);
		assert_ok!(Msa::add_key(owner_msa_id, &user_account_id, EMPTY_FUNCTION));

		let call_delete_msa_public_key: &<Test as frame_system::Config>::Call =
			&Call::Msa(MsaCall::delete_msa_public_key { key: owner_key_pair.public().into() });

		let info = DispatchInfo::default();
		let len = 0_usize;
		let result = CheckFreeExtrinsicUse::<Test>::new().validate(
			&owner_key_pair.public().into(),
			call_delete_msa_public_key,
			&info,
			len,
		);

		System::set_block_number(2);
		assert_ok!(result);
		assert_ok!(Msa::delete_msa_public_key(
			Origin::signed(AccountId32::from(owner_key_pair.public())),
			user_account_id.clone()
		));

		let call_delete_msa_public_key: &<Test as frame_system::Config>::Call =
			&Call::Msa(MsaCall::delete_msa_public_key { key: user_account_id.clone() });
		let info = DispatchInfo::default();
		let len = 0_usize;
		let result_deleted = CheckFreeExtrinsicUse::<Test>::new().validate(
			&user_account_id.clone(),
			call_delete_msa_public_key,
			&info,
			len,
		);
		assert!(result_deleted.is_err());
	});
}

/// Assert that when a key has been added to an MSA, that it my NOT be added to any other MSA.
/// Expected error: KeyAlreadyRegistered
#[test]
fn double_add_key_two_msa_fails() {
	new_test_ext().execute_with(|| {
		let (msa_id1, owner_key_pair) = create_account();
		let (_msa_id2, msa_2_owner_key_pair) = create_account();

		let add_new_key_data = AddKeyData { msa_id: msa_id1, expiration: 10 };
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let owner_signature: MultiSignature = owner_key_pair.sign(&encode_data_new_key_data).into();
		let new_key_signature: MultiSignature =
			msa_2_owner_key_pair.sign(&encode_data_new_key_data).into();

		assert_noop!(
			Msa::add_public_key_to_msa(
				test_origin_signed(1),
				owner_key_pair.public().into(),
				owner_signature,
				msa_2_owner_key_pair.public().into(),
				new_key_signature,
				add_new_key_data
			),
			Error::<Test>::KeyAlreadyRegistered
		);
	})
}

/// Assert that when a key has been deleted from one MSA, that it may be added to a different MSA.
#[test]
fn add_removed_key_to_msa_pass() {
	new_test_ext().execute_with(|| {
		let (msa_getting_a_second_key, owner_key_pair) = create_account();
		let (msa_used_to_have_a_key, prior_msa_key) = create_account();

		assert_ok!(Msa::delete_key_for_msa(msa_used_to_have_a_key, &prior_msa_key.public().into()));

		let add_new_key_data = AddKeyData { msa_id: msa_getting_a_second_key, expiration: 10 };
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());
		let owner_signature: MultiSignature = owner_key_pair.sign(&encode_data_new_key_data).into();
		let new_key_signature: MultiSignature =
			prior_msa_key.sign(&encode_data_new_key_data).into();
		assert_ok!(Msa::add_public_key_to_msa(
			test_origin_signed(1),
			owner_key_pair.public().into(),
			owner_signature,
			prior_msa_key.public().into(),
			new_key_signature,
			add_new_key_data
		));
	});
}

#[test]
fn create_provider() {
	new_test_ext().execute_with(|| {
		let (_new_msa_id, key_pair) = create_account();

		assert_ok!(Msa::create_provider(
			Origin::signed(key_pair.public().into()),
			Vec::from("Foo")
		));
	})
}

#[test]
fn create_provider_max_size_exceeded() {
	new_test_ext().execute_with(|| {
		let (_new_msa_id, key_pair) = create_account();

		assert_err!(
			Msa::create_provider(
				Origin::signed(key_pair.public().into()),
				Vec::from("12345678901234567")
			),
			Error::<Test>::ExceedsMaxProviderNameSize
		);
	})
}

#[test]
fn create_provider_duplicate() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let (_new_msa_id, _) =
			Msa::create_account(key_pair.public().into(), EMPTY_FUNCTION).unwrap();
		assert_ok!(Msa::create_provider(
			Origin::signed(key_pair.public().into()),
			Vec::from("Foo")
		));

		assert_err!(
			Msa::create_provider(Origin::signed(key_pair.public().into()), Vec::from("Foo")),
			Error::<Test>::DuplicateProviderRegistryEntry
		)
	})
}

fn set_schema_count<T: Config>(n: u16) {
	<T>::SchemaValidator::set_schema_count(n);
}

#[test]
pub fn error_invalid_schema_id_table() {
	struct TestCase<T> {
		schema: Vec<u16>,
		expected: T,
	}
	new_test_ext().execute_with(|| {
		set_schema_count::<Test>(12);

		let provider = Provider(1);
		let delegator = Delegator(2);
		let test_cases: [TestCase<Error<Test>>; 3] = [
			TestCase { schema: vec![15, 16], expected: Error::<Test>::InvalidSchemaId },
			TestCase { schema: vec![16, 17], expected: Error::<Test>::InvalidSchemaId },
			TestCase { schema: vec![18], expected: Error::<Test>::InvalidSchemaId },
		];
		for tc in test_cases {
			assert_noop!(Msa::add_provider(provider, delegator, tc.schema), tc.expected);
		}
	})
}

#[test]
pub fn error_exceeding_max_schema_under_minimum_schema_grants() {
	new_test_ext().execute_with(|| {
		set_schema_count::<Test>(16);

		let provider = Provider(1);
		let delegator = Delegator(2);
		assert_noop!(
			Msa::add_provider(provider, delegator, (1..32 as u16).collect::<Vec<_>>()),
			Error::<Test>::ExceedsMaxSchemaGrantsPerDelegation
		);
	})
}

#[test]
pub fn error_not_delegated_rpc() {
	new_test_ext().execute_with(|| {
		let provider = Provider(1);
		let delegator = Delegator(2);
		assert_err!(
			Msa::get_granted_schemas_by_msa_id(delegator, provider),
			Error::<Test>::DelegationNotFound
		);
	})
}

#[test]
pub fn error_schema_not_granted_rpc() {
	new_test_ext().execute_with(|| {
		let provider = Provider(1);
		let delegator = Delegator(2);
		assert_ok!(Msa::add_provider(provider, delegator, Vec::default()));
		assert_err!(
			Msa::get_granted_schemas_by_msa_id(delegator, provider),
			Error::<Test>::SchemaNotGranted
		);
	})
}

#[test]
pub fn schema_granted_success_rpc() {
	new_test_ext().execute_with(|| {
		set_schema_count::<Test>(2);

		let provider = Provider(1);
		let delegator = Delegator(2);
		let schema_grants = vec![1, 2];
		assert_ok!(Msa::add_provider(provider, delegator, schema_grants));
		let schemas_granted = Msa::get_granted_schemas_by_msa_id(delegator, provider);
		let expected_schemas_granted = vec![1, 2];
		let output_schemas: Vec<SchemaId> = schemas_granted.unwrap().unwrap();
		assert_eq!(output_schemas, expected_schemas_granted);
	})
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
		let call_delete_msa_public_key: &<Test as frame_system::Config>::Call =
			&Call::Msa(MsaCall::delete_msa_public_key { key: AccountId32::from(msa_new_key) });
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
		let call_revoke_delegation_by_delegator: &<Test as frame_system::Config>::Call =
			&Call::Msa(MsaCall::revoke_delegation_by_delegator { provider_msa_id });

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
		let pays_call_should_pass: &<Test as frame_system::Config>::Call =
			&Call::Msa(MsaCall::create {});

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
pub fn add_provider_expired() {
	new_test_ext().execute_with(|| {
		// 1. create two key pairs
		let (provider_pair, _) = sr25519::Pair::generate();
		let (user_pair, _) = sr25519::Pair::generate();

		let provider_key = provider_pair.public();
		let delegator_key = user_pair.public();

		// 2. create provider MSA
		assert_ok!(Msa::create(Origin::signed(provider_key.into()))); // MSA = 1

		// Register provider
		assert_ok!(Msa::create_provider(Origin::signed(provider_key.into()), Vec::from("Foo")));

		// 3. create delegator MSA and provider to provider
		let expiration: BlockNumber = 0;

		let add_provider_payload = AddProvider::new(1u64, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());
		let signature: MultiSignature = user_pair.sign(&encode_add_provider_data).into();
		// 3.5 create the user's MSA + add provider as provider
		assert_err!(
			Msa::grant_delegation(
				test_origin_signed(1),
				delegator_key.into(),
				signature,
				add_provider_payload
			),
			Error::<Test>::ProofHasExpired
		);
	})
}

#[test]
pub fn delegation_expired_long_back() {
	new_test_ext().execute_with(|| {
		let provider = Provider(1);
		let delegator = Delegator(2);

		assert_ok!(Msa::add_provider(provider, delegator, Vec::default()));

		System::set_block_number(System::block_number() + 100);
		assert_ok!(Msa::ensure_valid_delegation(provider, delegator, None));

		assert_ok!(Msa::revoke_provider(provider, delegator));

		System::set_block_number(System::block_number() + 150);

		assert_noop!(
			Msa::ensure_valid_delegation(provider, delegator, Some(151)),
			Error::<Test>::DelegationRevoked
		);
		assert_ok!(Msa::ensure_valid_delegation(provider, delegator, Some(6)));
		assert_noop!(
			Msa::ensure_valid_delegation(provider, delegator, Some(1000)),
			Error::<Test>::CannotPredictValidityPastCurrentBlock
		);
	})
}

#[test]
pub fn ensure_all_schema_ids_are_valid_errors() {
	new_test_ext().execute_with(|| {
		let schema_ids = vec![1];
		assert_noop!(
			Msa::ensure_all_schema_ids_are_valid(&schema_ids),
			Error::<Test>::InvalidSchemaId
		);

		let schema_ids = (1..32).collect::<Vec<_>>();
		assert_noop!(
			Msa::ensure_all_schema_ids_are_valid(&schema_ids),
			Error::<Test>::ExceedsMaxSchemaGrantsPerDelegation
		);
	})
}
#[test]
pub fn ensure_all_schema_ids_are_valid_success() {
	new_test_ext().execute_with(|| {
		let schema_ids = vec![1];
		set_schema_count::<Test>(1);

		assert_ok!(Msa::ensure_all_schema_ids_are_valid(&schema_ids));
	});
}

#[test]
pub fn is_registered_provider_is_true() {
	new_test_ext().execute_with(|| {
		let provider = Provider(1);
		let provider_name = Vec::from("frequency".as_bytes()).try_into().unwrap();

		let provider_meta = ProviderRegistryEntry { provider_name };
		ProviderToRegistryEntry::<Test>::insert(provider, provider_meta);

		assert!(Msa::is_registered_provider(provider.into()));
	});
}

fn register_signature_and_validate(
	current_block: BlockNumber,
	expected_bucket: u64,
	signature: &MultiSignature,
) {
	System::set_block_number(current_block as u64);
	let mortality_block = current_block + 111;
	assert_ok!(Msa::register_signature(signature, mortality_block.into()));

	let actual = <PayloadSignatureRegistry<Test>>::get(expected_bucket, signature);
	assert_eq!(Some(mortality_block as u64), actual);
}

#[test]
pub fn stores_signature_in_expected_bucket() {
	struct TestCase {
		current_block: BlockNumber,
		expected_bucket_number: u64,
	}

	new_test_ext().execute_with(|| {
		let test_cases: Vec<TestCase> = vec![
			TestCase { current_block: 999_899, expected_bucket_number: 0 }, // mortality = 1_000_010
			TestCase { current_block: 4_294_965_098, expected_bucket_number: 0 }, // mortality = 4_294_965_209
			TestCase { current_block: 0, expected_bucket_number: 0 },       // mortality = 111
			TestCase { current_block: 129, expected_bucket_number: 1 },     // mortality = 240
			TestCase { current_block: 640, expected_bucket_number: 1 },     // mortality = 751
			TestCase { current_block: 128_999_799, expected_bucket_number: 1 }, // mortality = 128_999_910
		];
		for tc in test_cases {
			// mortality block is current_block + 111 in this function.
			register_signature_and_validate(
				tc.current_block,
				tc.expected_bucket_number,
				&generate_test_signature(),
			);
		}
	})
}

#[test]
// for illustration purposes
pub fn bucket_for() {
	struct TestCase {
		block: u64,
		expected_bucket: u64,
	}
	new_test_ext().execute_with(|| {
		let test_cases: Vec<TestCase> = vec![
			TestCase { block: 1_010, expected_bucket: 1 },
			TestCase { block: 1_110, expected_bucket: 1 },
			TestCase { block: 1_201, expected_bucket: 0 },
			TestCase { block: 1_301, expected_bucket: 0 },
			TestCase { block: 1_401, expected_bucket: 1 },
			TestCase { block: 1_501, expected_bucket: 1 },
			TestCase { block: 1_601, expected_bucket: 0 },
			TestCase { block: 1_701, expected_bucket: 0 },
			TestCase { block: 1_801, expected_bucket: 1 },
			TestCase { block: 1_901, expected_bucket: 1 },
		];
		for tc in test_cases {
			assert_eq!(tc.expected_bucket, Msa::bucket_for(tc.block));
		}
	});
}

#[test]
pub fn clears_stale_signatures_after_mortality_limit() {
	new_test_ext().execute_with(|| {
		let sig1 = &generate_test_signature();
		let sig2 = &generate_test_signature();

		let mut current_block: BlockNumber = 667;
		let mortality_block = (current_block + 111) as u64;
		register_signature_and_validate(current_block, 1u64, sig1);
		register_signature_and_validate(current_block, 1u64, sig2);

		current_block = 777;
		run_to_block(current_block.into());
		// the old signature should not be able to be registered
		assert_noop!(
			Msa::register_signature(sig1, mortality_block),
			Error::<Test>::SignatureAlreadySubmitted
		);

		current_block = 876;
		run_to_block(current_block.into());

		assert_eq!(false, <PayloadSignatureRegistry<Test>>::contains_key(1u64, sig1));
		assert_eq!(false, <PayloadSignatureRegistry<Test>>::contains_key(1u64, sig2));
	})
}

#[test]
pub fn add_signature_replay_fails() {
	struct TestCase {
		current: u64,
		mortality: u64,
		run_to: u64,
	}
	new_test_ext().execute_with(|| {
		// these should all fail replay
		let test_cases: Vec<TestCase> = vec![
			TestCase { current: 10_849u64, mortality: 11_001u64, run_to: 11_000u64 }, // fails test
			TestCase { current: 1u64, mortality: 3u64, run_to: 2u64 },
			TestCase { current: 99u64, mortality: 101u64, run_to: 100u64 },
			TestCase { current: 1_000u64, mortality: 1_199u64, run_to: 1_198u64 },
			TestCase { current: 1_002u64, mortality: 1_201u64, run_to: 1_200u64 },
			TestCase { current: 999u64, mortality: 1_148u64, run_to: 1_101u64 },
		];
		for tc in test_cases {
			System::set_block_number(tc.current);
			let signature_new = &generate_test_signature();
			assert_ok!(Msa::register_signature(signature_new, tc.mortality));
			run_to_block(tc.run_to);
			assert_noop!(
				Msa::register_signature(signature_new, tc.mortality),
				Error::<Test>::SignatureAlreadySubmitted,
			);
		}
	});
}

#[test]
pub fn cannot_register_signature_with_mortality_out_of_bounds() {
	new_test_ext().execute_with(|| {
		System::set_block_number(11_122);
		let mut mortality_block: BlockNumber = 11_323;

		let sig1 = &generate_test_signature();
		assert_noop!(
			Msa::register_signature(sig1, mortality_block.into()),
			Error::<Test>::ProofNotYetValid
		);

		mortality_block = 11_122;
		assert_noop!(
			Msa::register_signature(sig1, mortality_block.into()),
			Error::<Test>::ProofHasExpired
		);
	})
}

#[test]
pub fn add_msa_key_replay_fails() {
	struct TestCase {
		current: u64,
		mortality: u32,
		run_to: u64,
		expected_ok: bool,
	}
	new_test_ext().execute_with(|| {
		// these should all fail replay
		let test_cases: Vec<TestCase> = vec![
			TestCase {
				current: 10_849u64,
				mortality: 11_001u32,
				run_to: 10_848u64,
				expected_ok: true,
			},
			TestCase { current: 1u64, mortality: 3u32, run_to: 5u64, expected_ok: false },
			TestCase { current: 99u64, mortality: 101u32, run_to: 100u64, expected_ok: true },
			TestCase {
				current: 1_000u64,
				mortality: 1_199u32,
				run_to: 1_198u64,
				expected_ok: true,
			},
			TestCase {
				current: 1_002u64,
				mortality: 1_201u32,
				run_to: 1_200u64,
				expected_ok: true,
			},
			TestCase { current: 999u64, mortality: 1_148u32, run_to: 1_101u64, expected_ok: true },
			TestCase {
				current: 1_000_000u64,
				mortality: 1_000_000u32,
				run_to: 1_000_000u64,
				expected_ok: false,
			},
		];

		let (new_msa_id, key_pair_provider) = create_account();
		let account_provider = key_pair_provider.public();
		for tc in test_cases {
			System::set_block_number(tc.current);
			let add_new_key_data = AddKeyData { msa_id: new_msa_id, expiration: tc.mortality };
			let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());
			let (new_key_pair, _) = sr25519::Pair::generate();
			let new_delegator_account = new_key_pair.public();
			let signature_owner: MultiSignature =
				key_pair_provider.sign(&encode_data_new_key_data).into();
			let signature_new_key: MultiSignature =
				new_key_pair.sign(&encode_data_new_key_data).into();
			run_to_block(tc.run_to);

			let add_key_response: bool = Msa::add_public_key_to_msa(
				Origin::signed(account_provider.into()),
				account_provider.into(),
				signature_owner.clone(),
				new_delegator_account.into(),
				signature_new_key,
				add_new_key_data.clone(),
			)
			.is_ok();

			assert_eq!(add_key_response, tc.expected_ok);
		}
	})
}

#[test]
fn grant_permissions_for_schemas_errors_when_no_delegation() {
	new_test_ext().execute_with(|| {
		let delegator = Delegator(2);
		let provider = Provider(1);
		let schema_ids = vec![1, 2];
		let result = Msa::grant_permissions_for_schemas(delegator, provider, schema_ids);

		assert_noop!(result, Error::<Test>::DelegationNotFound);
	});
}

#[test]
fn grant_permissions_for_schemas_errors_when_invalid_schema_id() {
	new_test_ext().execute_with(|| {
		set_schema_count::<Test>(1);
		let delegator = Delegator(2);
		let provider = Provider(1);
		let schema_grants = vec![1];

		assert_ok!(Msa::add_provider(provider, delegator, schema_grants));

		let additional_grants = vec![2];
		let result = Msa::grant_permissions_for_schemas(delegator, provider, additional_grants);

		assert_noop!(result, Error::<Test>::InvalidSchemaId);
	});
}

#[test]
fn grant_permissions_for_schemas_errors_when_exceeds_max_schema_grants() {
	new_test_ext().execute_with(|| {
		set_schema_count::<Test>(31);

		let delegator = Delegator(2);
		let provider = Provider(1);
		let schema_grants = vec![1];

		assert_ok!(Msa::add_provider(provider, delegator, schema_grants));

		let additional_grants = (2..32 as u16).collect::<Vec<_>>();
		let result = Msa::grant_permissions_for_schemas(delegator, provider, additional_grants);

		assert_noop!(result, Error::<Test>::ExceedsMaxSchemaGrantsPerDelegation);
	});
}

#[test]
fn grant_permissions_for_schema_success() {
	new_test_ext().execute_with(|| {
		set_schema_count::<Test>(3);

		let delegator = Delegator(2);
		let provider = Provider(1);
		let schema_grants = vec![1];

		assert_ok!(Msa::add_provider(provider, delegator, schema_grants));

		let delegation_relationship = Msa::get_delegation(delegator, provider).unwrap();
		let mut expected = BoundedBTreeMap::<
			SchemaId,
			<Test as frame_system::Config>::BlockNumber,
			<Test as Config>::MaxSchemaGrantsPerDelegation,
		>::new();

		expected.try_insert(1, Default::default()).expect("testing expected");

		assert_eq!(delegation_relationship.schema_permissions, expected);

		// Add new schema ids
		let additional_grants = vec![2];
		let result = Msa::grant_permissions_for_schemas(delegator, provider, additional_grants);

		assert_ok!(result);

		let delegation_relationship = Msa::get_delegation(delegator, provider).unwrap();
		let mut expected = BoundedBTreeMap::<
			SchemaId,
			<Test as frame_system::Config>::BlockNumber,
			<Test as Config>::MaxSchemaGrantsPerDelegation,
		>::new();

		expected.try_insert(1, Default::default()).expect("testing expected");
		expected.try_insert(2, Default::default()).expect("testing expected");

		assert_eq!(delegation_relationship.schema_permissions, expected);
	});
}

#[test]
fn grant_schema_permissions_errors_when_no_key_exists() {
	new_test_ext().execute_with(|| {
		let (delegator_pair, _) = sr25519::Pair::generate();
		let delegator_account = delegator_pair.public();

		let provider = Provider(2);
		let schema_ids: Vec<SchemaId> = vec![1];

		assert_noop!(
			Msa::grant_schema_permissions(
				Origin::signed(delegator_account.into()),
				provider.into(),
				schema_ids,
			),
			Error::<Test>::NoKeyExists
		);
	});
}

#[test]
fn grant_schema_permissions_errors_when_delegation_not_found_error() {
	new_test_ext().execute_with(|| {
		let (delegator_pair, _) = sr25519::Pair::generate();
		let delegator_account = delegator_pair.public();

		let provider = Provider(2);
		let schema_ids: Vec<SchemaId> = vec![1];

		assert_ok!(Msa::create(Origin::signed(delegator_account.into())));

		assert_noop!(
			Msa::grant_schema_permissions(
				Origin::signed(delegator_account.into()),
				provider.into(),
				schema_ids,
			),
			Error::<Test>::DelegationNotFound
		);
	});
}

#[test]
fn grant_schema_permissions_success() {
	new_test_ext().execute_with(|| {
		set_schema_count::<Test>(3);

		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let (delegator_pair, _) = sr25519::Pair::generate();
		let delegator_account = delegator_pair.public();

		assert_ok!(Msa::create(Origin::signed(delegator_account.into())));
		assert_ok!(Msa::create(Origin::signed(provider_account.into())));

		let delegator = Delegator(1);
		let provider = Provider(2);

		assert_ok!(Msa::add_provider(provider, delegator, Default::default()));

		let schema_ids: Vec<SchemaId> = vec![2];

		assert_ok!(Msa::grant_schema_permissions(
			Origin::signed(delegator_account.into()),
			provider.into(),
			schema_ids,
		));

		System::assert_last_event(Event::DelegationUpdated { provider, delegator }.into());
	});
}

#[test]
fn delegation_default_trait_impl() {
	new_test_ext().execute_with(|| {
		let delegation: Delegation<
			SchemaId,
			<Test as frame_system::Config>::BlockNumber,
			<Test as Config>::MaxSchemaGrantsPerDelegation,
		> = Default::default();

		let expected = Delegation {
			schema_permissions: BoundedBTreeMap::<
				SchemaId,
				<Test as frame_system::Config>::BlockNumber,
				<Test as Config>::MaxSchemaGrantsPerDelegation,
			>::default(),
			revoked_at: Default::default(),
		};

		assert_eq!(delegation, expected);
	});
}

#[test]
fn schema_permissions_trait_impl_try_insert_schema_success() {
	new_test_ext().execute_with(|| {
		let mut delegation: Delegation<
			SchemaId,
			<Test as frame_system::Config>::BlockNumber,
			<Test as Config>::MaxSchemaGrantsPerDelegation,
		> = Default::default();

		let schema_id = 1;
		assert_ok!(PermittedDelegationSchemas::<Test>::try_insert_schema(
			&mut delegation,
			schema_id
		));
		assert_eq!(delegation.schema_permissions.len(), 1);
	});
}

#[test]
fn schema_permissions_trait_impl_try_insert_schemas_errors_when_exceeds_max_schema_grants() {
	new_test_ext().execute_with(|| {
		let mut delegation: Delegation<
			SchemaId,
			<Test as frame_system::Config>::BlockNumber,
			<Test as Config>::MaxSchemaGrantsPerDelegation,
		> = Default::default();

		let schema_ids = (1..32).collect::<Vec<_>>();
		assert_noop!(
			PermittedDelegationSchemas::<Test>::try_insert_schemas(&mut delegation, schema_ids),
			Error::<Test>::ExceedsMaxSchemaGrantsPerDelegation
		);
	});
}

#[test]
fn try_mutate_delegation_success() {
	new_test_ext().execute_with(|| {
		let delegator = Delegator(1);
		let provider = Provider(2);

		assert_ok!(Msa::try_mutate_delegation(
			delegator,
			provider,
			|delegation, _is_new_provider| -> Result<(), &'static str> {
				let schema_id = 1;
				let _a =
					PermittedDelegationSchemas::<Test>::try_insert_schema(delegation, schema_id);

				Ok(())
			},
		));

		assert!(Msa::get_delegation(delegator, provider).is_some());
	});
}

#[test]
fn revoke_permissions_for_schema_success() {
	new_test_ext().execute_with(|| {
		set_schema_count::<Test>(3);

		let delegator = Delegator(2);
		let provider = Provider(1);
		let schema_grants = vec![1];

		assert_ok!(Msa::add_provider(provider, delegator, schema_grants));

		let delegation_relationship = Msa::get_delegation(delegator, provider).unwrap();
		let mut expected = BoundedBTreeMap::<
			SchemaId,
			<Test as frame_system::Config>::BlockNumber,
			<Test as Config>::MaxSchemaGrantsPerDelegation,
		>::new();

		expected.try_insert(1, Default::default()).expect("testing expected");

		assert_eq!(delegation_relationship.schema_permissions, expected);

		// Revoke schema ids
		let schemas_to_be_revoked = vec![1];
		let result =
			Msa::revoke_permissions_for_schemas(delegator, provider, schemas_to_be_revoked);

		assert_ok!(result);

		let delegation_relationship = Msa::get_delegation(delegator, provider).unwrap();
		let mut expected = BoundedBTreeMap::<
			SchemaId,
			<Test as frame_system::Config>::BlockNumber,
			<Test as Config>::MaxSchemaGrantsPerDelegation,
		>::new();

		expected.try_insert(1, 1u32.into()).expect("testing expected");

		assert_eq!(delegation_relationship.schema_permissions, expected);
	});
}

#[test]
fn revoke_permissions_for_schemas_errors_when_no_delegation() {
	new_test_ext().execute_with(|| {
		let delegator = Delegator(2);
		let provider = Provider(1);
		let schema_ids = vec![1, 2];
		let result = Msa::revoke_permissions_for_schemas(delegator, provider, schema_ids);

		assert_noop!(result, Error::<Test>::DelegationNotFound);
	});
}

#[test]
fn revoke_permissions_for_schemas_errors_when_schema_does_not_exist_in_list_of_schema_grants() {
	new_test_ext().execute_with(|| {
		set_schema_count::<Test>(31);

		let delegator = Delegator(2);
		let provider = Provider(1);
		let schema_grants = vec![1, 2];

		assert_ok!(Msa::add_provider(provider, delegator, schema_grants));

		let additional_grants = (3..32 as u16).collect::<Vec<_>>();
		let result = Msa::revoke_permissions_for_schemas(delegator, provider, additional_grants);

		assert_noop!(result, Error::<Test>::SchemaNotGranted);

		let result = Msa::get_delegation(delegator, provider);

		let mut expected = Delegation {
			revoked_at: 0u32.into(),
			schema_permissions: BoundedBTreeMap::<
				SchemaId,
				<Test as frame_system::Config>::BlockNumber,
				<Test as Config>::MaxSchemaGrantsPerDelegation,
			>::new(),
		};

		expected
			.schema_permissions
			.try_insert(1, 0u32.into())
			.expect("testing expected");

		expected
			.schema_permissions
			.try_insert(2, 0u32.into())
			.expect("testing expected");

		assert_eq!(result.unwrap(), expected);
	});
}

#[test]
fn revoke_schema_permissions_success() {
	new_test_ext().execute_with(|| {
		set_schema_count::<Test>(3);

		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let (delegator_pair, _) = sr25519::Pair::generate();
		let delegator_account = delegator_pair.public();

		assert_ok!(Msa::create(Origin::signed(delegator_account.into())));
		assert_ok!(Msa::create(Origin::signed(provider_account.into())));

		let delegator = Delegator(1);
		let provider = Provider(2);

		assert_ok!(Msa::add_provider(provider, delegator, vec![1, 2]));

		let schema_ids_to_revoke: Vec<SchemaId> = vec![2];

		assert_ok!(Msa::revoke_schema_permissions(
			Origin::signed(delegator_account.into()),
			provider.into(),
			schema_ids_to_revoke,
		));

		System::assert_last_event(Event::DelegationUpdated { provider, delegator }.into());
	});
}

#[test]
fn revoke_schema_permissions_errors_when_no_key_exists() {
	new_test_ext().execute_with(|| {
		let (delegator_pair, _) = sr25519::Pair::generate();
		let delegator_account = delegator_pair.public();

		let provider = Provider(2);
		let schema_ids: Vec<SchemaId> = vec![1];

		assert_noop!(
			Msa::revoke_schema_permissions(
				Origin::signed(delegator_account.into()),
				provider.into(),
				schema_ids,
			),
			Error::<Test>::NoKeyExists
		);
	});
}

#[test]
fn schema_permissions_trait_impl_try_get_mut_schema_success() {
	new_test_ext().execute_with(|| {
		let mut delegation: Delegation<
			SchemaId,
			<Test as frame_system::Config>::BlockNumber,
			<Test as Config>::MaxSchemaGrantsPerDelegation,
		> = Default::default();

		let schema_id = 1;
		assert_ok!(PermittedDelegationSchemas::<Test>::try_insert_schema(
			&mut delegation,
			schema_id
		));
		let default_block_number = 0u64;

		assert_eq!(delegation.schema_permissions.len(), 1);
		assert_eq!(delegation.schema_permissions.get(&schema_id).unwrap(), &default_block_number);

		let revoked_block_number = 2u64;

		assert_ok!(PermittedDelegationSchemas::<Test>::try_get_mut_schema(
			&mut delegation,
			schema_id,
			revoked_block_number.clone()
		));

		assert_eq!(delegation.schema_permissions.get(&schema_id).unwrap(), &revoked_block_number);
	});
}

#[test]
pub fn ensure_valid_schema_grant_success() {
	new_test_ext().execute_with(|| {
		set_schema_count::<Test>(2);

		let provider = Provider(1);
		let delegator = Delegator(2);
		let schema_grants = vec![1, 2];
		assert_ok!(Msa::add_provider(provider, delegator, schema_grants));

		System::set_block_number(System::block_number() + 1);

		assert_ok!(Msa::ensure_valid_schema_grant(provider, delegator, 1_u16, 1u64));
	})
}

#[test]
pub fn ensure_valid_schema_grant_errors_when_delegation_relationship_is_valid_and_grant_does_not_exist(
) {
	new_test_ext().execute_with(|| {
		set_schema_count::<Test>(2);

		let provider = Provider(1);
		let delegator = Delegator(2);
		let schema_grants = vec![1, 2];

		// Add delegation relationship with schema grants
		assert_ok!(Msa::add_provider(provider, delegator, schema_grants));

		System::set_block_number(System::block_number() + 1);
		assert_eq!(System::block_number(), 2);

		assert_err!(
			Msa::ensure_valid_schema_grant(provider, delegator, 3_u16, 1u64),
			Error::<Test>::SchemaNotGranted
		);
	})
}

#[test]
pub fn ensure_valid_schema_grant_errors_when_delegation_relationship_is_valid_and_schema_grant_is_revoked(
) {
	new_test_ext().execute_with(|| {
		set_schema_count::<Test>(2);

		// create delegation relationship
		let provider = Provider(1);
		let delegator = Delegator(2);
		let schema_grants = vec![1, 2];
		assert_ok!(Msa::add_provider(provider, delegator, schema_grants));

		// create delegation relationship
		System::set_block_number(System::block_number() + 5);
		assert_eq!(System::block_number(), 6);

		// revoke schema permission at block 6
		assert_ok!(Msa::revoke_permissions_for_schemas(delegator, provider, vec![1]));

		// Schemas is valid for the current block that is revoked 6
		assert_ok!(Msa::ensure_valid_schema_grant(provider, delegator, 1, 6));

		// Checking that asking for validity past the current block, 6, errors
		assert_noop!(
			Msa::ensure_valid_schema_grant(provider, delegator, 1, 7),
			Error::<Test>::CannotPredictValidityPastCurrentBlock
		);

		System::set_block_number(System::block_number() + 5);
		assert_eq!(System::block_number(), 11);

		assert_noop!(
			Msa::ensure_valid_schema_grant(provider, delegator, 1, 7),
			Error::<Test>::SchemaNotGranted
		);
	});
}

#[test]
pub fn ensure_valid_schema_grant_errors_delegation_revoked_when_delegation_relationship_has_been_revoked(
) {
	new_test_ext().execute_with(|| {
		// Set the schemas counts so that it passes validation
		set_schema_count::<Test>(2);

		// create delegation relationship
		let provider = Provider(1);
		let delegator = Delegator(2);
		let schema_grants = vec![1, 2];

		// create delegation relationship
		assert_ok!(Msa::add_provider(provider, delegator, schema_grants));

		// move forward to block 6
		System::set_block_number(System::block_number() + 5);
		assert_eq!(System::block_number(), 6);

		// revoke delegation relationship at block 6
		assert_ok!(Msa::revoke_provider(provider, delegator));

		// Schemas is valid for the current block that is revoked 6
		assert_ok!(Msa::ensure_valid_schema_grant(provider, delegator, 1, 6));
		assert_ok!(Msa::ensure_valid_schema_grant(provider, delegator, 1, 5));

		// Checking that asking for validity past the current block, 6, errors
		assert_noop!(
			Msa::ensure_valid_schema_grant(provider, delegator, 1, 7),
			Error::<Test>::CannotPredictValidityPastCurrentBlock
		);

		System::set_block_number(System::block_number() + 5);
		assert_eq!(System::block_number(), 11);

		// Check that schema is not valid after delegation revocation
		assert_noop!(
			Msa::ensure_valid_schema_grant(provider, delegator, 1, 7),
			Error::<Test>::DelegationRevoked
		);
	});
}
