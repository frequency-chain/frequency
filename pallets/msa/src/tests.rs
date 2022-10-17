use frame_support::{
	assert_err, assert_noop, assert_ok,
	weights::{DispatchInfo, GetDispatchInfo, Pays, Weight},
};
use sp_core::{crypto::AccountId32, sr25519, Encode, Pair, H256};
use sp_runtime::{traits::SignedExtension, MultiSignature};

use common_primitives::{
	msa::{Delegator, MessageSourceId, OrderedSetExt, Provider, ProviderInfo},
	node::BlockNumber,
	schema::SchemaId,
	utils::wrap_binary_data,
};
use common_runtime::extensions::check_nonce::CheckNonce;

use crate::{
	ensure,
	mock::*,
	types::{AddKeyData, AddProvider, EMPTY_FUNCTION},
	CheckFreeExtrinsicUse, Config, DispatchResult, Error, Event, MsaIdentifier,
	PayloadSignatureRegistry,
};

#[test]
fn it_creates_an_msa_account() {
	new_test_ext().execute_with(|| {
		assert_ok!(Msa::create(test_origin_signed(1)));

		assert_eq!(Msa::get_msa_by_account_id(test_public(1)), Some(1 as MessageSourceId));

		assert_eq!(Msa::get_identifier(), 1);

		System::assert_last_event(Event::MsaCreated { msa_id: 1, key: test_public(1) }.into());
	});
}

#[test]
fn it_throws_msa_identifier_overflow() {
	new_test_ext().execute_with(|| {
		MsaIdentifier::<Test>::set(u64::MAX);

		assert_noop!(Msa::create(test_origin_signed(1)), Error::<Test>::MsaIdOverflow);
	});
}

#[test]
#[allow(unused_must_use)]
fn it_does_not_allow_duplicate_keys() {
	new_test_ext().execute_with(|| {
		Msa::create(test_origin_signed(1));

		assert_noop!(Msa::create(test_origin_signed(1)), Error::<Test>::KeyAlreadyRegistered);

		assert_eq!(Msa::get_identifier(), 1);
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
		let (key_pair, _) = sr25519::Pair::generate();
		let (key_pair_2, _) = sr25519::Pair::generate();

		let new_account = key_pair.public();
		let (new_msa_id, _) = Msa::create_account(new_account.into(), EMPTY_FUNCTION).unwrap();

		let fake_account = key_pair_2.public();

		let add_new_key_data = AddKeyData { nonce: 1, msa_id: new_msa_id, expiration: 10 };
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let signature: MultiSignature = key_pair.sign(&encode_data_new_key_data).into();

		assert_noop!(
			Msa::add_key_to_msa(
				test_origin_signed(1),
				fake_account.into(),
				signature,
				add_new_key_data
			),
			Error::<Test>::AddKeySignatureVerificationFailed
		);
	});
}

#[test]
fn it_throws_error_when_not_msa_owner() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let (key_pair_2, _) = sr25519::Pair::generate();

		let account = key_pair.public();

		let (new_msa_id, _) = Msa::create_account(account.into(), EMPTY_FUNCTION).unwrap();
		assert_ok!(Msa::create_account(test_public(1), EMPTY_FUNCTION));

		let new_account = key_pair_2.public();

		let add_new_key_data = AddKeyData { nonce: 0, msa_id: new_msa_id, expiration: 10 };
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let signature: MultiSignature = key_pair_2.sign(&encode_data_new_key_data).into();

		assert_noop!(
			Msa::add_key_to_msa(
				test_origin_signed(1),
				new_account.into(),
				signature,
				add_new_key_data
			),
			Error::<Test>::NotMsaOwner
		);
	});
}

#[test]
fn it_throws_error_when_for_duplicate_key() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();

		let new_account = key_pair.public();

		let (new_msa_id, _) = Msa::create_account(new_account.into(), EMPTY_FUNCTION).unwrap();

		let add_new_key_data = AddKeyData { nonce: 1, msa_id: new_msa_id, expiration: 10 };
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let signature: MultiSignature = key_pair.sign(&encode_data_new_key_data).into();

		assert_noop!(
			Msa::add_key_to_msa(
				Origin::signed(new_account.into()),
				new_account.into(),
				signature,
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
		let (key_pair, _) = sr25519::Pair::generate();
		let account = key_pair.public();
		let (new_msa_id, _) = Msa::create_account(account.into(), EMPTY_FUNCTION).unwrap();
		let add_new_key_data = AddKeyData { nonce: 1, msa_id: new_msa_id, expiration: 10 };
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		for _ in 1..<Test as Config>::MaxKeys::get() {
			let (new_key_pair, _) = sr25519::Pair::generate();
			let new_account = new_key_pair.public();
			let signature: MultiSignature = new_key_pair.sign(&encode_data_new_key_data).into();
			assert_ok!(Msa::add_key_to_msa(
				Origin::signed(account.into()),
				new_account.into(),
				signature,
				add_new_key_data.clone()
			));
		}

		// act
		let (final_key_pair, _) = sr25519::Pair::generate();
		let final_account = final_key_pair.public();
		let signature: MultiSignature = final_key_pair.sign(&encode_data_new_key_data).into();
		assert_noop!(
			Msa::add_key_to_msa(
				Origin::signed(account.into()),
				final_account.into(),
				signature,
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
		let (key_pair, _) = sr25519::Pair::generate();
		let (key_pair_2, _) = sr25519::Pair::generate();

		let account = key_pair.public();
		let (new_msa_id, _) = Msa::create_account(account.into(), EMPTY_FUNCTION).unwrap();

		let new_key = key_pair_2.public();

		let add_new_key_data = AddKeyData { nonce: 1, msa_id: new_msa_id, expiration: 10 };
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let signature: MultiSignature = key_pair_2.sign(&encode_data_new_key_data).into();

		// act
		assert_ok!(Msa::add_key_to_msa(
			Origin::signed(account.into()),
			new_key.into(),
			signature,
			add_new_key_data,
		));

		// assert
		// *Temporarily Removed* until https://github.com/LibertyDSNP/frequency/issues/418// *Temporarily Removed* until https://github.com/LibertyDSNP/frequency/issues/418
		// let keys = Msa::fetch_msa_keys(new_msa_id);
		// assert_eq!(keys.len(), 2);
		// assert_eq!{keys.contains(&KeyInfoResponse {key: AccountId32::from(new_key), msa_id: new_msa_id}), true}

		let keys_count = Msa::get_msa_key_count(new_msa_id);
		assert_eq!(keys_count, 2);
		System::assert_last_event(Event::KeyAdded { msa_id: 1, key: new_key.into() }.into());
	});
}

/// Assert that when attempting to add a key to an MSA with an expired proof that the key is NOT added.
/// Expected error: ProofHasExpired
#[test]
fn add_key_with_expired_proof_fails() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let (key_pair_2, _) = sr25519::Pair::generate();

		let account = key_pair.public();
		let (new_msa_id, _) = Msa::create_account(account.into(), EMPTY_FUNCTION).unwrap();

		let new_key = key_pair_2.public();

		// The current block is 1, therefore setting the proof expiration to 1 shoud cause
		// the extrinsic to fail because the proof has expired.
		let add_new_key_data = AddKeyData { nonce: 1, msa_id: new_msa_id, expiration: 1 };
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		System::set_block_number(2);

		let signature: MultiSignature = key_pair_2.sign(&encode_data_new_key_data).into();

		assert_noop!(
			Msa::add_key_to_msa(
				Origin::signed(account.into()),
				new_key.into(),
				signature,
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
		let (key_pair, _) = sr25519::Pair::generate();
		let (key_pair_2, _) = sr25519::Pair::generate();

		let account = key_pair.public();
		let (new_msa_id, _) = Msa::create_account(account.into(), EMPTY_FUNCTION).unwrap();

		let new_key = key_pair_2.public();

		// The current block is 1, therefore setting the proof expiration to  + 1
		// should cause the extrinsic to fail because the proof is only valid for
		// more blocks.
		let add_new_key_data = AddKeyData { nonce: 1, msa_id: new_msa_id, expiration: 202 };
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let signature: MultiSignature = key_pair_2.sign(&encode_data_new_key_data).into();

		assert_noop!(
			Msa::add_key_to_msa(
				Origin::signed(account.into()),
				new_key.into(),
				signature,
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

		assert_ok!(Msa::delete_msa_key(test_origin_signed(1), test_public(2)));

		let info = Msa::get_msa_by_account_id(&test_public(2));

		assert_eq!(info, None);

		System::assert_last_event(Event::KeyRemoved { key: test_public(2) }.into());
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
		assert_noop!(Msa::delete_msa_key(origin, test_account), Error::<Test>::InvalidSelfRemoval);
	})
}

#[test]
fn test_retire_msa_success() {
	new_test_ext().execute_with(|| {
		let (test_account_key_pair, _) = sr25519::Pair::generate();
		let msa_id = 2;

		// Create an account
		let test_account = AccountId32::new(test_account_key_pair.public().into());
		let origin = Origin::signed(test_account.clone());

		// Add an account to the MSA so it has exactly one account
		assert_ok!(Msa::add_key(msa_id, &test_account, EMPTY_FUNCTION));

		// Retire the MSA
		assert_ok!(Msa::retire_msa(origin));

		// Check if MsaRetired event was dispatched.
		System::assert_last_event(Event::MsaRetired { msa_id }.into());

		// Assert that the MSA has no accounts
		let key_count = Msa::get_msa_key_count(msa_id);
		assert_eq!(key_count, 0);

		// MSA has been retired, perform additional tests

		// [TEST] Adding an account to the retired MSA should fail
		let (key_pair1, _) = sr25519::Pair::generate();
		let new_account1 = key_pair1.public();
		let (key_pair2, _) = sr25519::Pair::generate();
		let new_account2 = key_pair2.public();
		let (msa_id2, _) = Msa::create_account(new_account2.into(), EMPTY_FUNCTION).unwrap();

		let add_new_key_data = AddKeyData { nonce: 1, msa_id: msa_id2, expiration: 10 };
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());
		let signature: MultiSignature = key_pair1.sign(&encode_data_new_key_data).into();
		assert_noop!(
			Msa::add_key_to_msa(
				Origin::signed(test_account.clone()),
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
			Msa::try_get_msa_from_account_id(&AccountId32::new(provider_account.0)).unwrap();

		// Register provider
		assert_ok!(Msa::register_provider(
			Origin::signed(provider_account.into()),
			Vec::from("Foo")
		));

		let (delegator_signature, add_provider_payload) =
			create_and_sign_add_provider_payload(test_account_key_pair, provider_msa_id);

		assert_noop!(
			Msa::add_provider_to_msa(
				Origin::signed(provider_account.into()),
				test_account.clone(),
				delegator_signature,
				add_provider_payload
			),
			Error::<Test>::NoKeyExists
		);

		// [TEST] Revoking a provider (modifying permissions) should fail
		assert_noop!(
			Msa::revoke_msa_delegation_by_delegator(
				Origin::signed(test_account.clone()),
				provider_msa_id
			),
			Error::<Test>::NoKeyExists
		);
	})
}

#[test]
fn test_retire_msa_fails_if_registered_provider() {
	new_test_ext().execute_with(|| {
		// Add an account to the MSA
		assert_ok!(Msa::add_key(2, &test_public(1), EMPTY_FUNCTION));

		// Register provider
		assert_ok!(Msa::register_provider(test_origin_signed(1), Vec::from("Foo")));

		// Retire MSA
		assert_noop!(
			Msa::retire_msa(test_origin_signed(1)),
			Error::<Test>::RegisteredProviderCannotBeRetired
		);
	})
}

#[test]
fn test_retire_msa_fails_if_more_than_one_account_exists() {
	new_test_ext().execute_with(|| {
		// Add an account to the MSA
		assert_ok!(Msa::add_key(2, &test_public(1), EMPTY_FUNCTION));
		// Add an account to the MSA
		assert_ok!(Msa::add_key(2, &test_public(2), EMPTY_FUNCTION));

		// Retire the MSA
		assert_noop!(Msa::retire_msa(test_origin_signed(1)), Error::<Test>::MoreThanOneKeyExists);
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

		let info = Msa::get_msa_by_account_id(&test_public(1));

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
			Msa::try_get_msa_from_account_id(&AccountId32::new(provider_account.0)).unwrap();

		// Create delegator account and get its MSA ID (u64)
		assert_ok!(Msa::create(Origin::signed(delegator_account.into())));
		let delegator_msa =
			Msa::try_get_msa_from_account_id(&AccountId32::new(delegator_account.0)).unwrap();

		// Register provider
		assert_ok!(Msa::register_provider(
			Origin::signed(provider_account.into()),
			Vec::from("Foo")
		));

		let (delegator_signature, add_provider_payload) =
			create_and_sign_add_provider_payload(delegator_pair, provider_msa);

		assert_ok!(Msa::add_provider_to_msa(
			Origin::signed(provider_account.into()),
			delegator_account.into(),
			delegator_signature,
			add_provider_payload
		));

		let provider = Provider(provider_msa);
		let delegator = Delegator(delegator_msa);

		assert_eq!(
			Msa::get_provider_info(delegator, provider),
			Some(ProviderInfo { expired: 0, schemas: OrderedSet::new() })
		);

		System::assert_last_event(
			Event::ProviderAdded { delegator: delegator_msa.into(), provider: provider_msa.into() }
				.into(),
		);
	});
}

#[test]
pub fn add_provider_to_msa_throws_add_provider_verification_failed() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let account = key_pair.public();
		let expiration: BlockNumber = 10;
		let add_provider_payload = AddProvider::new(2, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = key_pair.sign(&encode_add_provider_data).into();
		let fake_provider_payload = AddProvider::new(3, None, expiration);
		assert_noop!(
			Msa::add_provider_to_msa(
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
pub fn add_provider_to_msa_throws_no_key_exist_error() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let expiration: BlockNumber = 10;
		let add_provider_payload = AddProvider::new(2, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = key_pair.sign(&encode_add_provider_data).into();

		assert_noop!(
			Msa::add_provider_to_msa(
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
pub fn add_provider_to_msa_throws_key_revoked_error() {
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
			Msa::add_provider_to_msa(
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
pub fn add_provider_to_msa_throws_invalid_self_provider_error() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let expiration: BlockNumber = 10;
		let add_provider_payload = AddProvider::new(1, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());
		let signature: MultiSignature = key_pair.sign(&encode_add_provider_data).into();

		assert_ok!(Msa::create(Origin::signed(provider_account.into())));

		assert_noop!(
			Msa::add_provider_to_msa(
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
pub fn add_provider_to_msa_throws_unauthorized_delegator_error() {
	new_test_ext().execute_with(|| {
		// Generate a key pair for the provider
		let (provider_key_pair, _) = sr25519::Pair::generate();
		let provider_account = provider_key_pair.public();

		// Generate a key pair for the delegator
		let (delegator_key_pair, _) = sr25519::Pair::generate();
		let delegator_account = delegator_key_pair.public();
		assert_ok!(Msa::create(Origin::signed(delegator_account.into())));
		let delegator_msa_id =
			Msa::try_get_msa_from_account_id(&AccountId32::new(delegator_account.0)).unwrap();

		let expiration: BlockNumber = 10;
		let add_provider_payload = AddProvider::new(delegator_msa_id, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());
		let signature: MultiSignature = delegator_key_pair.sign(&encode_add_provider_data).into();

		assert_ok!(Msa::create(Origin::signed(provider_account.into())));

		// Register provider
		assert_ok!(Msa::register_provider(
			Origin::signed(provider_account.into()),
			Vec::from("Foo")
		));

		assert_noop!(
			Msa::add_provider_to_msa(
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
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let (key_pair_delegator, _) = sr25519::Pair::generate();
		let delegator_account = key_pair_delegator.public();

		let expiration: BlockNumber = 10;

		let add_provider_payload = AddProvider::new(1u64, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = key_pair_delegator.sign(&encode_add_provider_data).into();

		assert_ok!(Msa::create(Origin::signed(provider_account.into())));

		// Register provider
		assert_ok!(Msa::register_provider(
			Origin::signed(provider_account.into()),
			Vec::from("Foo")
		));

		// act
		assert_ok!(Msa::create_sponsored_account_with_delegation(
			Origin::signed(provider_account.into()),
			delegator_account.into(),
			signature,
			add_provider_payload
		));

		// assert
		let key_info = Msa::get_msa_by_account_id(&AccountId32::new(delegator_account.0));
		assert_eq!(key_info.unwrap(), 2);

		let provider_info = Msa::get_provider_info(Delegator(2), Provider(1));
		assert_eq!(provider_info.is_some(), true);

		let events_occured = System::events();
		// let provider_registered_event = &events_occured.as_slice()[1];
		let created_event = &events_occured.as_slice()[2];
		let provider_event = &events_occured.as_slice()[3];
		assert_eq!(
			created_event.event,
			Event::MsaCreated { msa_id: 2u64, key: delegator_account.into() }.into()
		);
		assert_eq!(
			provider_event.event,
			Event::ProviderAdded { provider: 1u64.into(), delegator: 2u64.into() }.into()
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
		assert_ok!(Msa::register_provider(
			Origin::signed(provider_account.into()),
			Vec::from("Foo")
		));

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
		assert_ok!(Msa::register_provider(
			Origin::signed(provider_account.into()),
			Vec::from("Foo")
		));

		// act
		assert_err!(
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
		assert_eq!(Msa::get_msa_by_account_id(&key), None);

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
pub fn revoke_msa_delegation_by_delegator_is_successful() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let (delegator_pair, _) = sr25519::Pair::generate();
		let delegator_account = delegator_pair.public();

		assert_ok!(Msa::create(Origin::signed(delegator_account.into())));
		assert_ok!(Msa::create(Origin::signed(provider_account.into())));

		// Register provider
		assert_ok!(Msa::register_provider(
			Origin::signed(provider_account.into()),
			Vec::from("Foo")
		));

		let provider_msa =
			Msa::try_get_msa_from_account_id(&AccountId32::new(provider_account.0)).unwrap();

		let (delegator_signature, add_provider_payload) =
			create_and_sign_add_provider_payload(delegator_pair, provider_msa);

		assert_ok!(Msa::add_provider_to_msa(
			Origin::signed(provider_account.into()),
			delegator_account.into(),
			delegator_signature,
			add_provider_payload
		));

		assert_ok!(Msa::revoke_msa_delegation_by_delegator(
			Origin::signed(delegator_account.into()),
			2
		));

		System::assert_last_event(
			Event::DelegatorRevokedDelegation { delegator: 1.into(), provider: 2.into() }.into(),
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
			Msa::try_get_msa_from_account_id(&AccountId32::new(provider_account.0)).unwrap();

		let (delegator_signature, add_provider_payload) =
			create_and_sign_add_provider_payload(delegator_pair, provider_msa);

		// Register provider
		assert_ok!(Msa::register_provider(
			Origin::signed(provider_account.into()),
			Vec::from("Foo")
		));

		assert_ok!(Msa::add_provider_to_msa(
			Origin::signed(provider_account.into()),
			delegator_account.into(),
			delegator_signature,
			add_provider_payload
		));

		let delegator_msa =
			Msa::try_get_msa_from_account_id(&AccountId32::new(delegator_account.0)).unwrap();

		let provider = Provider(provider_msa);
		let delegator = Delegator(delegator_msa);

		assert_ok!(Msa::revoke_provider(provider, delegator));

		assert_eq!(
			Msa::get_provider_info(delegator, provider).unwrap(),
			ProviderInfo { expired: 1, schemas: OrderedSet::new() },
		);
	});
}

#[test]
fn revoke_msa_delegation_by_delegator_fails_when_no_msa() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Msa::revoke_msa_delegation_by_delegator(test_origin_signed(1), 1),
			Error::<Test>::NoKeyExists
		);
	});
}

#[test]
pub fn revoke_msa_delegation_fails_if_only_key_is_revoked() {
	new_test_ext().execute_with(|| {
		assert_ok!(Msa::create(test_origin_signed(2)));
		assert_ok!(Msa::delete_key_for_msa(1, &test_public(2)));
		assert_noop!(
			Msa::revoke_msa_delegation_by_delegator(test_origin_signed(2), 1),
			Error::<Test>::NoKeyExists
		);
	})
}

#[test]
pub fn revoke_msa_delegation_by_delegator_fails_if_has_msa_but_no_delegation() {
	new_test_ext().execute_with(|| {
		assert_ok!(Msa::create(test_origin_signed(1)));
		assert_ok!(Msa::create(test_origin_signed(2)));
		assert_noop!(
			Msa::revoke_msa_delegation_by_delegator(test_origin_signed(1), 2),
			Error::<Test>::DelegationNotFound
		);
	})
}

#[test]
fn revoke_provider_throws_error_when_delegation_already_revoked() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let (delegator_pair, _) = sr25519::Pair::generate();
		let delegator_account = delegator_pair.public();

		assert_ok!(Msa::create(Origin::signed(delegator_account.into())));
		assert_ok!(Msa::create(Origin::signed(provider_account.into())));

		let provider_msa =
			Msa::try_get_msa_from_account_id(&AccountId32::new(provider_account.0)).unwrap();

		let (delegator_signature, add_provider_payload) =
			create_and_sign_add_provider_payload(delegator_pair, provider_msa);

		// Register provider
		assert_ok!(Msa::register_provider(
			Origin::signed(provider_account.into()),
			Vec::from("Foo")
		));

		assert_ok!(Msa::add_provider_to_msa(
			Origin::signed(provider_account.into()),
			delegator_account.into(),
			delegator_signature,
			add_provider_payload
		));

		assert_ok!(Msa::revoke_msa_delegation_by_delegator(
			Origin::signed(delegator_account.into()),
			provider_msa
		));

		assert_noop!(
			Msa::revoke_msa_delegation_by_delegator(
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
		assert_ok!(Msa::register_provider(test_origin_signed(1), Vec::from("Foo")));

		assert_ok!(Msa::add_provider_to_msa(
			test_origin_signed(1),
			provider_account.into(),
			signature,
			add_provider_payload
		));

		let call = MsaCall::<Test>::revoke_msa_delegation_by_delegator { provider_msa_id: 2 };
		let dispatch_info = call.get_dispatch_info();

		assert_eq!(dispatch_info.pays_fee, Pays::No);
	})
}

#[test]
pub fn revoke_provider_throws_delegation_not_found_error() {
	new_test_ext().execute_with(|| {
		// 1. create two key pairs
		let (provider_pair, _) = sr25519::Pair::generate();
		let (user_pair, _) = sr25519::Pair::generate();
		let provider_key = provider_pair.public();
		let delegator_key = user_pair.public();

		assert_ok!(Msa::create(Origin::signed(provider_key.into())));
		// 1. error when delegator msa_id not found
		assert_noop!(
			Msa::revoke_delegation_by_provider(Origin::signed(provider_key.into()), 2u64),
			Error::<Test>::DelegationNotFound
		);

		assert_ok!(Msa::create(Origin::signed(delegator_key.into())));
		// 2. error when no delegation relationship
		assert_noop!(
			Msa::revoke_delegation_by_provider(Origin::signed(provider_key.into()), 2u64),
			Error::<Test>::DelegationNotFound
		);

		Error::<Test>::DelegationNotFound
	});
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
		assert_ok!(Msa::register_provider(Origin::signed(provider_key.into()), Vec::from("Foo")));

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
		let provider_info = Msa::get_provider_info(Delegator(2), Provider(1));
		assert_eq!(provider_info, Some(ProviderInfo { expired: 26, schemas: OrderedSet::new() }));

		// 7. verify the event
		System::assert_last_event(
			Event::ProviderRevokedDelegation { provider: Provider(1), delegator: Delegator(2) }
				.into(),
		);
	})
}

#[test]
pub fn remove_msa_delegation_call_has_correct_costs() {
	new_test_ext().execute_with(|| {
		let call = MsaCall::<Test>::revoke_delegation_by_provider { delegator: 2 };
		let dispatch_info = call.get_dispatch_info();

		assert_eq!(dispatch_info.pays_fee, Pays::No);
	})
}

#[test]
pub fn revoke_delegation_by_provider_errors_when_no_delegator_msa_id() {
	new_test_ext().execute_with(|| {
		let (provider_pair, _) = sr25519::Pair::generate();
		let (user_pair, _) = sr25519::Pair::generate();

		let provider_key = provider_pair.public();
		let delegator_key = user_pair.public();

		// 0. when provider msa_id not found
		assert_noop!(
			Msa::revoke_delegation_by_provider(Origin::signed(provider_key.into()), 2u64),
			Error::<Test>::NoKeyExists
		);

		assert_ok!(Msa::create(Origin::signed(provider_key.into())));

		System::set_block_number(System::block_number() + 19);

		// 1. when delegator msa_id not found
		assert_noop!(
			Msa::revoke_delegation_by_provider(Origin::signed(provider_key.into()), 2u64),
			Error::<Test>::DelegationNotFound
		);

		assert_ok!(Msa::create(Origin::signed(delegator_key.into())));
		// 2. when no delegation relationship
		assert_noop!(
			Msa::revoke_delegation_by_provider(Origin::signed(provider_key.into()), 2u64),
			Error::<Test>::DelegationNotFound
		);

		assert_ok!(Msa::add_provider(Provider(1), Delegator(2), Vec::default()));
		assert_ok!(Msa::revoke_provider(Provider(1), Delegator(2)));
		// 3. when_delegation_expired
		assert_noop!(
			Msa::revoke_delegation_by_provider(Origin::signed(provider_key.into()), 2u64),
			Error::<Test>::DelegationRevoked
		);
	})
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
			Error::<Test>::DelegationExpired
		);
	})
}

/// Assert that revoking an MSA delegation passes the signed extension CheckFreeExtrinsicUse
/// validation when a valid delegation exists.
#[test]
fn signed_extension_revoke_msa_delegation_by_delegator() {
	new_test_ext().execute_with(|| {
		let (provider_msa_id, delegator_account) = test_create_delegator_msa_with_provider();
		let call_revoke_delegation: &<Test as frame_system::Config>::Call =
			&Call::Msa(MsaCall::revoke_msa_delegation_by_delegator { provider_msa_id });
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
fn signed_extension_validation_failure_on_revoked() {
	new_test_ext().execute_with(|| {
		let (provider_msa_id, delegator_account) = test_create_delegator_msa_with_provider();
		let call_revoke_delegation: &<Test as frame_system::Config>::Call =
			&Call::Msa(MsaCall::revoke_msa_delegation_by_delegator { provider_msa_id });
		let info = DispatchInfo::default();
		let len = 0_usize;
		let result = CheckFreeExtrinsicUse::<Test>::new().validate(
			&delegator_account.into(),
			call_revoke_delegation,
			&info,
			len,
		);
		assert_ok!(result);
		assert_ok!(Msa::revoke_msa_delegation_by_delegator(
			Origin::signed(delegator_account.into()),
			provider_msa_id
		));

		System::set_block_number(System::block_number() + 1);
		let call_revoke_delegation: &<Test as frame_system::Config>::Call =
			&Call::Msa(MsaCall::revoke_msa_delegation_by_delegator { provider_msa_id });
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

/// Assert that a call that is not revoke_msa_delegation_by_delegator passes the signed extension
/// CheckFreeExtrinsicUse validaton.
#[test]
fn signed_extension_validation_valid_for_others() {
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
pub fn delete_msa_key_call_has_correct_costs() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let new_key = key_pair.public();

		let call = MsaCall::<Test>::delete_msa_key { key: AccountId32::from(new_key) };
		let dispatch_info = call.get_dispatch_info();
		assert_eq!(dispatch_info.pays_fee, Pays::No);
	})
}

#[test]
fn signed_extension_validation_on_msa_key_deleted() {
	new_test_ext().execute_with(|| {
		let (owner_msa_id, owner_key) = create_account();

		let (user_key_pair, _) = sr25519::Pair::generate();
		let user_public_key = user_key_pair.public();
		let user_account_id = AccountId32::from(user_public_key);
		assert_ok!(Msa::add_key(owner_msa_id, &user_account_id, EMPTY_FUNCTION));

		let call_delete_msa_key: &<Test as frame_system::Config>::Call =
			&Call::Msa(MsaCall::delete_msa_key { key: owner_key.clone() });

		let info = DispatchInfo::default();
		let len = 0_usize;
		let result = CheckFreeExtrinsicUse::<Test>::new().validate(
			&owner_key,
			call_delete_msa_key,
			&info,
			len,
		);
		assert_ok!(result);
		assert_ok!(Msa::delete_msa_key(
			Origin::signed(AccountId32::from(owner_key.clone())),
			user_account_id
		));
	});
}

#[test]
fn signed_extension_validation_failure_on_msa_key_deleted() {
	new_test_ext().execute_with(|| {
		let (owner_msa_id, owner_key) = create_account();

		let (user_key_pair, _) = sr25519::Pair::generate();
		let user_public_key = user_key_pair.public();
		let user_account_id = AccountId32::from(user_public_key);
		assert_ok!(Msa::add_key(owner_msa_id, &user_account_id, EMPTY_FUNCTION));

		let call_delete_msa_key: &<Test as frame_system::Config>::Call =
			&Call::Msa(MsaCall::delete_msa_key { key: owner_key.clone() });

		let info = DispatchInfo::default();
		let len = 0_usize;
		let result = CheckFreeExtrinsicUse::<Test>::new().validate(
			&owner_key,
			call_delete_msa_key,
			&info,
			len,
		);

		System::set_block_number(2);
		assert_ok!(result);
		assert_ok!(Msa::delete_msa_key(
			Origin::signed(AccountId32::from(owner_key.clone())),
			user_account_id.clone()
		));

		let call_delete_msa_key: &<Test as frame_system::Config>::Call =
			&Call::Msa(MsaCall::delete_msa_key { key: user_account_id.clone() });
		let info = DispatchInfo::default();
		let len = 0_usize;
		let result_deleted = CheckFreeExtrinsicUse::<Test>::new().validate(
			&user_account_id.clone(),
			call_delete_msa_key,
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
		let (key_pair1, _) = sr25519::Pair::generate();
		let new_account1 = key_pair1.public();
		let (key_pair2, _) = sr25519::Pair::generate();
		let new_account2 = key_pair2.public();
		let (_msa_id1, _) = Msa::create_account(new_account1.into(), EMPTY_FUNCTION).unwrap();
		let (msa_id2, _) = Msa::create_account(new_account2.into(), EMPTY_FUNCTION).unwrap();

		let add_new_key_data = AddKeyData { nonce: 1, msa_id: msa_id2, expiration: 10 };
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());
		let signature: MultiSignature = key_pair1.sign(&encode_data_new_key_data).into();
		assert_noop!(
			Msa::add_key_to_msa(
				Origin::signed(new_account2.into()),
				new_account1.into(),
				signature,
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
		let (key_pair1, _) = sr25519::Pair::generate();
		let new_account1 = key_pair1.public();
		let (key_pair2, _) = sr25519::Pair::generate();
		let new_account2 = key_pair2.public();
		let (msa_id1, _) = Msa::create_account(new_account1.into(), EMPTY_FUNCTION).unwrap();
		let (msa_id2, _) = Msa::create_account(new_account2.into(), EMPTY_FUNCTION).unwrap();

		assert_ok!(Msa::delete_key_for_msa(msa_id1, &new_account1.into()));

		let add_new_key_data = AddKeyData { nonce: 1, msa_id: msa_id2, expiration: 10 };
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());
		let signature: MultiSignature = key_pair1.sign(&encode_data_new_key_data).into();
		assert_ok!(Msa::add_key_to_msa(
			Origin::signed(new_account2.into()),
			new_account1.into(),
			signature,
			add_new_key_data
		));
	});
}

#[test]
fn register_provider() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let (_new_msa_id, _) =
			Msa::create_account(key_pair.public().into(), EMPTY_FUNCTION).unwrap();
		assert_ok!(Msa::register_provider(
			Origin::signed(key_pair.public().into()),
			Vec::from("Foo")
		));
	})
}

#[test]
fn register_provider_max_size_exceeded() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let (_new_msa_id, _) =
			Msa::create_account(key_pair.public().into(), EMPTY_FUNCTION).unwrap();
		assert_err!(
			Msa::register_provider(
				Origin::signed(key_pair.public().into()),
				Vec::from("12345678901234567")
			),
			Error::<Test>::ExceedsMaxProviderNameSize
		);
	})
}

#[test]
fn register_provider_duplicate() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let (_new_msa_id, _) =
			Msa::create_account(key_pair.public().into(), EMPTY_FUNCTION).unwrap();
		assert_ok!(Msa::register_provider(
			Origin::signed(key_pair.public().into()),
			Vec::from("Foo")
		));

		assert_err!(
			Msa::register_provider(Origin::signed(key_pair.public().into()), Vec::from("Foo")),
			Error::<Test>::DuplicateProviderMetadata
		)
	})
}

#[test]
pub fn valid_schema_grant() {
	new_test_ext().execute_with(|| {
		let provider = Provider(1);
		let delegator = Delegator(2);
		let schemas: Vec<SchemaId> = vec![1, 2];
		assert_ok!(Msa::add_provider(provider, delegator, schemas));

		System::set_block_number(System::block_number() + 1);

		assert_ok!(Msa::ensure_valid_schema_grant(provider, delegator, 1_u16));
	})
}

#[test]
pub fn error_exceeding_max_schema_grants() {
	new_test_ext().execute_with(|| {
		let provider = Provider(1);
		let delegator = Delegator(2);
		let schemas: Vec<SchemaId> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
		assert_err!(
			Msa::add_provider(provider, delegator, schemas),
			Error::<Test>::ExceedsMaxSchemaGrants
		);
	})
}

#[test]
pub fn error_schema_not_granted() {
	new_test_ext().execute_with(|| {
		let provider = Provider(1);
		let delegator = Delegator(2);
		let schemas: Vec<SchemaId> = vec![1, 2];
		assert_ok!(Msa::add_provider(provider, delegator, schemas));

		System::set_block_number(System::block_number() + 1);

		assert_err!(
			Msa::ensure_valid_schema_grant(provider, delegator, 3_u16),
			Error::<Test>::SchemaNotGranted
		);
	})
}

#[test]
pub fn error_not_delegated_rpc() {
	new_test_ext().execute_with(|| {
		let provider = Provider(1);
		let delegator = Delegator(2);
		assert_err!(
			Msa::get_granted_schemas(delegator, provider),
			Error::<Test>::DelegationNotFound
		);
	})
}

#[test]
pub fn error_schema_not_granted_rpc() {
	new_test_ext().execute_with(|| {
		let provider = Provider(1);
		let delegator = Delegator(2);
		assert_ok!(Msa::add_provider(provider, delegator, vec![]));
		assert_err!(Msa::get_granted_schemas(delegator, provider), Error::<Test>::SchemaNotGranted);
	})
}

#[test]
pub fn schema_granted_success_rpc() {
	new_test_ext().execute_with(|| {
		let provider = Provider(1);
		let delegator = Delegator(2);
		let schemas: Vec<SchemaId> = vec![1, 2];
		assert_ok!(Msa::add_provider(provider, delegator, schemas));
		let schemas_granted = Msa::get_granted_schemas(delegator, provider);
		let expected_schemas_granted = vec![1, 2];
		let output_schemas: Vec<SchemaId> = schemas_granted.unwrap().unwrap();
		assert_eq!(output_schemas, expected_schemas_granted);
	})
}

// This scenario must fail:
// 1. User creates MSA and delegates to provider
// 2. User revokes msa delegation
// 3. User adds a key to their msa
// 4. User deletes first key from msa
// 5. Provider successfully calls "create_sponsored_account_with_delegation"
#[test]
pub fn replaying_create_sponsored_account_with_delegation_fails() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_key = key_pair.public();

		let (key_pair_delegator, _) = sr25519::Pair::generate();
		let delegator_key = key_pair_delegator.public();

		let expiration: BlockNumber = 100;
		let add_provider_payload = AddProvider::new(1u64, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());
		let signature: MultiSignature = key_pair_delegator.sign(&encode_add_provider_data).into();

		// create MSA for provider and register them
		assert_ok!(Msa::create(Origin::signed(provider_key.into())));
		assert_ok!(Msa::register_provider(Origin::signed(provider_key.into()), Vec::from("Foo")));

		// Step 1
		assert_ok!(Msa::create_sponsored_account_with_delegation(
			Origin::signed(provider_key.into()),
			delegator_key.into(),
			signature.clone(),
			add_provider_payload.clone()
		));

		// Step 2
		assert_ok!(Msa::revoke_msa_delegation_by_delegator(
			Origin::signed(delegator_key.into()),
			1
		));
		// Step 3
		let (key_pair_delegator2, _) = sr25519::Pair::generate();
		let delegator_account2 = key_pair_delegator2.public();

		let add_key_payload: AddKeyData = AddKeyData { msa_id: 2, nonce: 0, expiration: 110 };
		let encode_add_key_data = wrap_binary_data(add_key_payload.encode());
		let add_key_signature = key_pair_delegator2.sign(&encode_add_key_data);

		assert_ok!(Msa::add_key_to_msa(
			Origin::signed(delegator_key.into()),
			delegator_account2.into(),
			add_key_signature.into(),
			add_key_payload
		));
		assert_ok!(Msa::delete_msa_key(
			Origin::signed(delegator_account2.into()),
			delegator_key.into(),
		));

		// expect call create with same signature to fail
		assert_err!(
			Msa::create_sponsored_account_with_delegation(
				Origin::signed(provider_key.into()),
				delegator_key.into(),
				signature.clone(),
				add_provider_payload.clone(),
			),
			Error::<Test>::SignatureAlreadySubmitted
		);

		// expect this to fail for the same reason
		assert_err!(
			Msa::add_provider_to_msa(
				Origin::signed(provider_key.into()),
				delegator_key.into(),
				signature.clone(),
				add_provider_payload.clone(),
			),
			Error::<Test>::SignatureAlreadySubmitted
		);
	})
}

// This scenario should fail:
//   1. provider authorizes being added as provider to MSA and MSA account adds them.
//   2. provider removes them as MSA (say by quickly discovering MSA is undesirable)
//   3. MSA account replays the add, using the previous signed payload + signature.
#[ignore]
#[test]
fn replaying_add_provider_to_msa_fails() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_key = key_pair.public();

		let (key_pair_delegator, _) = sr25519::Pair::generate();
		let delegator_key = key_pair_delegator.public();

		// add_provider_payload in this case has delegator's msa_id as authorized_msa_id
		let expiration: BlockNumber = 10;
		let add_provider_payload = AddProvider::new(1u64, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		// DELEGATOR signs to add the provider
		let signature: MultiSignature = key_pair_delegator.sign(&encode_add_provider_data).into();

		// create MSA for provider and register them
		assert_ok!(Msa::create(Origin::signed(provider_key.into())));
		assert_ok!(Msa::register_provider(Origin::signed(provider_key.into()), Vec::from("Foo")));

		// create MSA for delegator
		assert_ok!(Msa::create(Origin::signed(delegator_key.into())));

		assert_ok!(Msa::add_provider_to_msa(
			Origin::signed(provider_key.into()),
			delegator_key.into(),
			signature.clone(),
			add_provider_payload.clone(),
		));

		// provider revokes the delegation.
		assert_ok!(Msa::revoke_delegation_by_provider(Origin::signed(provider_key.into()), 2));
		System::set_block_number(System::block_number() + 1);

		// Expected to fail because revoking the delegation just expires it at a given block number.
		assert_err!(
			Msa::add_provider_to_msa(
				Origin::signed(provider_key.into()),
				delegator_key.into(),
				signature.clone(),
				add_provider_payload.clone(),
			),
			Error::<Test>::SignatureAlreadySubmitted
		);
	})
}

// Assert that check nonce validation does not create a token account for delete_msa_key call.
#[test]
fn signed_ext_check_nonce_delete_msa_key() {
	new_test_ext().execute_with(|| {
		// Generate a key pair for MSA account
		let (msa_key_pair, _) = sr25519::Pair::generate();
		let msa_new_key = msa_key_pair.public();

		let len = 0_usize;

		// Test the delete_msa_key() call
		let call_delete_msa_key: &<Test as frame_system::Config>::Call =
			&Call::Msa(MsaCall::delete_msa_key { key: AccountId32::from(msa_new_key) });
		let info = call_delete_msa_key.get_dispatch_info();

		// Call delete_msa_key() using the Alice account
		let who = test_public(1);
		assert_ok!(CheckNonce::<Test>(0).pre_dispatch(&who, call_delete_msa_key, &info, len));

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

// Assert that check nonce validation does not create a token account for revoke_msa_delegation_by_delegator call.
#[test]
fn signed_ext_check_nonce_revoke_msa_delegation_by_delegator() {
	new_test_ext().execute_with(|| {
		let (provider_msa_id, _) = test_create_delegator_msa_with_provider();

		// We are testing the revoke_msa_delegation_by_delegator() call.
		let call_revoke_msa_delegation_by_delegator: &<Test as frame_system::Config>::Call =
			&Call::Msa(MsaCall::revoke_msa_delegation_by_delegator { provider_msa_id });

		let len = 0_usize;

		// Get the dispatch info for the call.
		let info = call_revoke_msa_delegation_by_delegator.get_dispatch_info();

		// Call revoke_msa_delegation_by_delegator() using the Alice account
		let who = test_public(1);
		assert_ok!(CheckNonce::<Test>(0).pre_dispatch(
			&who,
			call_revoke_msa_delegation_by_delegator,
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
		assert_ok!(Msa::register_provider(Origin::signed(provider_key.into()), Vec::from("Foo")));

		// 3. create delegator MSA and provider to provider
		let expiration: BlockNumber = 0;

		let add_provider_payload = AddProvider::new(1u64, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());
		let signature: MultiSignature = user_pair.sign(&encode_add_provider_data).into();
		// 3.5 create the user's MSA + add provider as provider
		assert_err!(
			Msa::add_provider_to_msa(
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
			Error::<Test>::DelegationExpired
		);
		assert_ok!(Msa::ensure_valid_delegation(provider, delegator, Some(6)));
		assert_noop!(
			Msa::ensure_valid_delegation(provider, delegator, Some(1000)),
			Error::<Test>::DelegationNotFound
		);
	})
}

fn generate_test_signature() -> MultiSignature {
	let (key_pair, _) = sr25519::Pair::generate();
	let fake_data = H256::random();
	key_pair.sign(fake_data.as_bytes()).into()
}

#[test]
pub fn register_signature_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(11_233);

		let mortality_block: BlockNumber = 11_243;
		let sig1 = &generate_test_signature();
		assert_ok!(Msa::register_signature(sig1, mortality_block.into()));
	})
}

#[test]
pub fn stores_signature_with_correct_key() {
	struct TestCase {
		current_block: BlockNumber,
		expected_bucket_number: u64,
	}

	new_test_ext().execute_with(|| {
		let test_cases: Vec<TestCase> = vec![
			TestCase { current_block: 129, expected_bucket_number: 0 },
			TestCase { current_block: 999_899, expected_bucket_number: 0 },
			TestCase { current_block: 4_294_965_098, expected_bucket_number: 0 },
			TestCase { current_block: 0, expected_bucket_number: 1 },
			TestCase { current_block: 640, expected_bucket_number: 1 },
			TestCase { current_block: 128_999_799, expected_bucket_number: 1 },
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
pub fn adds_new_bucket_number_mortality_to_store() {
	struct TestCase {
		current_block: BlockNumber,
		mortality_block: BlockNumber,
		// u64 instead of BlockNUmber is because of the conflict between BlockNumber and Header in mock;
		// BlockNumber has to be u64 so that Header will work, but BlockNumber is u32 in Msa Config
		expected_bucket: u64,
		expected_mortality: u64,
	}

	new_test_ext().execute_with(|| {
		let test_cases: Vec<TestCase> = vec![
			TestCase {
				current_block: 0,
				mortality_block: 99,
				expected_bucket: 0,
				expected_mortality: 100,
			},
			TestCase {
				current_block: 33_111,
				mortality_block: 33_260,
				expected_bucket: 0,
				expected_mortality: 33_300,
			},
			TestCase {
				current_block: 299,
				mortality_block: 399,
				expected_bucket: 1,
				expected_mortality: 400,
			},
			TestCase {
				current_block: 1222,
				mortality_block: 1322,
				expected_bucket: 1,
				expected_mortality: 1400,
			},
		];
		for tc in test_cases {
			System::set_block_number(tc.current_block as u64);
			let sig = &generate_test_signature();
			assert_ok!(Msa::register_signature(sig, tc.mortality_block.into()));
			assert_eq!(
				Some(tc.expected_mortality),
				Msa::get_mortality_block_of(tc.expected_bucket)
			);
		}
	})
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
pub fn clears_stale_signatures_after_mortality_limit() {
	new_test_ext().execute_with(|| {
		let sig1 = &generate_test_signature();
		let sig2 = &generate_test_signature();
		let sig3 = &generate_test_signature();

		let mut current_block: BlockNumber = 667;
		register_signature_and_validate(current_block, 1u64, sig1);
		register_signature_and_validate(current_block, 1u64, sig2);

		current_block += 100;
		register_signature_and_validate(current_block, 0u64, sig2);

		current_block += 100;
		let mortality_block = current_block + 111;
		System::set_block_number(current_block as u64);
		// the old signature can't be re-registered, and does not trigger a clear.
		assert_noop!(
			Msa::register_signature(sig1, mortality_block as u64),
			Error::<Test>::SignatureAlreadySubmitted
		);

		// a new signature triggers a clear.
		register_signature_and_validate(current_block, 1u64, sig3);
		// 1_sig1 and 1_sig2 should no longer be there
		assert_eq!(false, <PayloadSignatureRegistry<Test>>::contains_key(1u64, sig1));
		assert_eq!(false, <PayloadSignatureRegistry<Test>>::contains_key(1u64, sig2));
	})
}

#[test]
pub fn cannot_add_signature_twice() {
	new_test_ext().execute_with(|| {
		System::set_block_number(11_122);
		let mortality_block: BlockNumber = 11_321;

		let sig1 = &generate_test_signature();
		assert_ok!(Msa::register_signature(sig1, mortality_block.into()));

		assert_noop!(
			Msa::register_signature(sig1, mortality_block.into()),
			Error::<Test>::SignatureAlreadySubmitted
		);
	})
}

#[test]
pub fn cannot_add_signature_with_mortality_block_too_high() {
	new_test_ext().execute_with(|| {
		System::set_block_number(11_122);
		let mortality_block: BlockNumber = 11_323;

		let sig1 = &generate_test_signature();
		assert_noop!(
			Msa::register_signature(sig1, mortality_block.into()),
			Error::<Test>::ProofNotYetValid
		);
	})
}
