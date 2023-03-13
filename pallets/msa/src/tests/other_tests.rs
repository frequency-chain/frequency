use frame_support::{
	assert_err, assert_noop, assert_ok,
	dispatch::{GetDispatchInfo, Pays},
	BoundedBTreeMap,
};

use sp_core::{crypto::AccountId32, sr25519, Encode, Pair};
use sp_runtime::{ArithmeticError, MultiSignature};

use crate::{
	ensure,
	tests::mock::*,
	types::{AddKeyData, AddProvider, PermittedDelegationSchemas, EMPTY_FUNCTION},
	Config, DispatchResult, Error, Event, ProviderToRegistryEntry,
};

use common_primitives::{
	msa::{
		Delegation, DelegatorId, MessageSourceId, ProviderId, ProviderRegistryEntry,
		SchemaGrantValidator, SignatureRegistryPointer,
	},
	node::BlockNumber,
	schema::{SchemaId, SchemaValidator},
	utils::wrap_binary_data,
};

pub fn assert_revoke_delegation_by_delegator_no_effect(
	test_account: AccountId32,
	provider_msa_id: u64,
) {
	let event_count = System::event_count();
	assert_ok!(Msa::revoke_delegation_by_delegator(
		RuntimeOrigin::signed(test_account.clone()),
		provider_msa_id
	));
	assert_eq!(event_count, System::event_count())
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
fn it_throws_error_when_new_key_verification_fails() {
	new_test_ext().execute_with(|| {
		let (new_msa_id, owner_key_pair) = create_account();
		let (new_key_pair, _) = sr25519::Pair::generate();
		let (fake_key_pair, _) = sr25519::Pair::generate();

		let add_new_key_data = AddKeyData {
			msa_id: new_msa_id,
			expiration: 10,
			new_public_key: new_key_pair.public().into(),
		};

		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let owner_signature: MultiSignature = owner_key_pair.sign(&encode_data_new_key_data).into();
		let fake_new_key_signature: MultiSignature =
			fake_key_pair.sign(&encode_data_new_key_data).into();

		assert_noop!(
			Msa::add_public_key_to_msa(
				test_origin_signed(1),
				owner_key_pair.public().into(),
				owner_signature,
				fake_new_key_signature,
				add_new_key_data
			),
			Error::<Test>::NewKeyOwnershipInvalidSignature
		);
	});
}

#[test]
fn it_throws_error_when_msa_ownership_verification_fails() {
	new_test_ext().execute_with(|| {
		let (new_msa_id, owner_key_pair) = create_account();
		let (new_key_pair, _) = sr25519::Pair::generate();
		let (fake_owner_pair, _) = sr25519::Pair::generate();

		let add_new_key_data = AddKeyData {
			msa_id: new_msa_id,
			expiration: 10,
			new_public_key: new_key_pair.public().into(),
		};

		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let fake_owner_signature: MultiSignature =
			fake_owner_pair.sign(&encode_data_new_key_data).into();
		let new_key_signature: MultiSignature = new_key_pair.sign(&encode_data_new_key_data).into();

		assert_noop!(
			Msa::add_public_key_to_msa(
				test_origin_signed(1),
				owner_key_pair.public().into(),
				fake_owner_signature,
				new_key_signature,
				add_new_key_data
			),
			Error::<Test>::MsaOwnershipInvalidSignature
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

		let add_new_key_data = AddKeyData {
			msa_id: new_msa_id,
			expiration: 10,
			new_public_key: new_key_pair.public().into(),
		};

		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let fake_owner_signature: MultiSignature =
			fake_key_pair.sign(&encode_data_new_key_data).into();
		let new_key_signature: MultiSignature = new_key_pair.sign(&encode_data_new_key_data).into();

		assert_noop!(
			Msa::add_public_key_to_msa(
				test_origin_signed(1),
				fake_key_pair.public().into(),
				fake_owner_signature,
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

		let add_new_key_data = AddKeyData {
			msa_id: new_msa_id,
			expiration: 10,
			new_public_key: new_key_pair.public().into(),
		};

		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let owner_signature: MultiSignature = owner_key_pair.sign(&encode_data_new_key_data).into();
		let new_key_signature: MultiSignature = new_key_pair.sign(&encode_data_new_key_data).into();

		assert_noop!(
			Msa::add_public_key_to_msa(
				test_origin_signed(1),
				owner_key_pair.public().into(),
				owner_signature,
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

			let add_new_key_data = AddKeyData {
				msa_id: new_msa_id,
				expiration: 10,
				new_public_key: new_key_pair.public().into(),
			};
			let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

			let owner_signature: MultiSignature =
				owner_key_pair.sign(&encode_data_new_key_data).into();

			let public_key_ownership_signature =
				new_key_pair.sign(&encode_data_new_key_data).into();

			assert_ok!(Msa::add_public_key_to_msa(
				test_origin_signed(1),
				owner_key_pair.public().into(),
				owner_signature,
				public_key_ownership_signature,
				add_new_key_data
			));
		}

		// act
		let (final_key_pair, _) = sr25519::Pair::generate();

		let add_new_key_data = AddKeyData {
			msa_id: new_msa_id,
			expiration: 10,
			new_public_key: final_key_pair.public().into(),
		};
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let owner_signature: MultiSignature = owner_key_pair.sign(&encode_data_new_key_data).into();
		let new_key_signature: MultiSignature =
			final_key_pair.sign(&encode_data_new_key_data).into();

		assert_noop!(
			Msa::add_public_key_to_msa(
				test_origin_signed(1),
				owner_key_pair.public().into(),
				owner_signature,
				new_key_signature,
				add_new_key_data
			),
			ArithmeticError::Overflow
		);
	});
}

#[test]
fn add_key_with_valid_request_should_store_value_and_event() {
	new_test_ext().execute_with(|| {
		// arrange
		let (new_msa_id, owner_key_pair) = create_account();
		let (new_key_pair, _) = sr25519::Pair::generate();

		let add_new_key_data = AddKeyData {
			msa_id: new_msa_id,
			expiration: 10,
			new_public_key: new_key_pair.public().into(),
		};
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let owner_signature: MultiSignature = owner_key_pair.sign(&encode_data_new_key_data).into();
		let new_key_signature: MultiSignature = new_key_pair.sign(&encode_data_new_key_data).into();

		// act
		assert_ok!(Msa::add_public_key_to_msa(
			test_origin_signed(1),
			owner_key_pair.public().into(),
			owner_signature,
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
		let add_new_key_data = AddKeyData {
			msa_id: new_msa_id,
			expiration: 1,
			new_public_key: new_key_pair.public().into(),
		};

		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let owner_signature: MultiSignature = owner_key_pair.sign(&encode_data_new_key_data).into();
		let new_key_signature: MultiSignature = new_key_pair.sign(&encode_data_new_key_data).into();

		assert_noop!(
			Msa::add_public_key_to_msa(
				test_origin_signed(1),
				owner_key_pair.public().into(),
				owner_signature,
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
		let add_new_key_data = AddKeyData {
			msa_id: new_msa_id,
			expiration: 202,
			new_public_key: new_key_pair.public().into(),
		};

		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let owner_signature: MultiSignature = owner_key_pair.sign(&encode_data_new_key_data).into();
		let new_key_signature: MultiSignature = new_key_pair.sign(&encode_data_new_key_data).into();

		assert_noop!(
			Msa::add_public_key_to_msa(
				test_origin_signed(1),
				owner_key_pair.public().into(),
				owner_signature,
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
		assert_ok!(Msa::create(RuntimeOrigin::signed(provider_account.into())));
		let provider_msa =
			Msa::ensure_valid_msa_key(&AccountId32::new(provider_account.0)).unwrap();

		// Create delegator account and get its MSA ID (u64)
		assert_ok!(Msa::create(RuntimeOrigin::signed(delegator_account.into())));
		let delegator_msa =
			Msa::ensure_valid_msa_key(&AccountId32::new(delegator_account.0)).unwrap();

		// Register provider
		assert_ok!(Msa::create_provider(
			RuntimeOrigin::signed(provider_account.into()),
			Vec::from("Foo")
		));

		let (delegator_signature, add_provider_payload) =
			create_and_sign_add_provider_payload(delegator_pair, provider_msa);

		set_schema_count::<Test>(10);

		assert_ok!(Msa::grant_delegation(
			RuntimeOrigin::signed(provider_account.into()),
			delegator_account.into(),
			delegator_signature,
			add_provider_payload
		));

		let provider = ProviderId(provider_msa);
		let delegator = DelegatorId(delegator_msa);

		assert_eq!(
			Msa::get_delegation(delegator, provider),
			Some(Delegation { revoked_at: 0, schema_permissions: Default::default() })
		);

		System::assert_last_event(
			Event::DelegationGranted {
				delegator_id: delegator_msa.into(),
				provider_id: provider_msa.into(),
			}
			.into(),
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
		assert_ok!(Msa::create_provider(
			RuntimeOrigin::signed(provider_account.into()),
			Vec::from("Foo")
		));

		let (key_pair_delegator, _) = sr25519::Pair::generate();
		let delegator_account = key_pair_delegator.public();

		let expiration: BlockNumber = 10;

		let add_provider_payload = AddProvider::new(provider_msa, None, expiration);
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = key_pair_delegator.sign(&encode_add_provider_data).into();

		// act
		assert_ok!(Msa::create_sponsored_account_with_delegation(
			RuntimeOrigin::signed(provider_account.into()),
			delegator_account.into(),
			signature,
			add_provider_payload
		));

		// assert
		let delegator_msa =
			Msa::get_msa_by_public_key(&AccountId32::new(delegator_account.0)).unwrap();

		let provider_info = Msa::get_delegation(DelegatorId(2), ProviderId(1));
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
				provider_id: provider_msa.into(),
				delegator_id: delegator_msa.into()
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

		assert_ok!(Msa::create(RuntimeOrigin::signed(provider_account.into())));

		// act
		assert_noop!(
			Msa::create_sponsored_account_with_delegation(
				RuntimeOrigin::signed(provider_account.into()),
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

		assert_ok!(Msa::create(RuntimeOrigin::signed(provider_account.into())));
		assert_ok!(Msa::create(RuntimeOrigin::signed(delegator_account.into())));

		// Register provider
		assert_ok!(Msa::create_provider(
			RuntimeOrigin::signed(provider_account.into()),
			Vec::from("Foo")
		));

		// act
		assert_noop!(
			Msa::create_sponsored_account_with_delegation(
				RuntimeOrigin::signed(provider_account.into()),
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

		assert_ok!(Msa::create(RuntimeOrigin::signed(provider_account.into())));

		// act
		assert_noop!(
			Msa::create_sponsored_account_with_delegation(
				RuntimeOrigin::signed(provider_account.into()),
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

		assert_ok!(Msa::create(RuntimeOrigin::signed(provider_account.into())));

		// Register provider
		assert_ok!(Msa::create_provider(
			RuntimeOrigin::signed(provider_account.into()),
			Vec::from("Foo")
		));

		// act
		assert_noop!(
			Msa::create_sponsored_account_with_delegation(
				RuntimeOrigin::signed(provider_account.into()),
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

		assert_ok!(Msa::create(RuntimeOrigin::signed(delegator_account.into())));
		assert_ok!(Msa::create(RuntimeOrigin::signed(provider_account.into())));

		// Register provider
		assert_ok!(Msa::create_provider(
			RuntimeOrigin::signed(provider_account.into()),
			Vec::from("Foo")
		));

		let provider_msa =
			Msa::ensure_valid_msa_key(&AccountId32::new(provider_account.0)).unwrap();

		let (delegator_signature, add_provider_payload) =
			create_and_sign_add_provider_payload(delegator_pair, provider_msa);

		assert_ok!(Msa::grant_delegation(
			RuntimeOrigin::signed(provider_account.into()),
			delegator_account.into(),
			delegator_signature,
			add_provider_payload
		));

		assert_ok!(Msa::revoke_delegation_by_delegator(
			RuntimeOrigin::signed(delegator_account.into()),
			2
		));

		System::assert_last_event(
			Event::DelegationRevoked { delegator_id: 1.into(), provider_id: 2.into() }.into(),
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

		assert_ok!(Msa::create(RuntimeOrigin::signed(delegator_account.into())));
		assert_ok!(Msa::create(RuntimeOrigin::signed(provider_account.into())));

		let provider_msa =
			Msa::ensure_valid_msa_key(&AccountId32::new(provider_account.0)).unwrap();

		let (delegator_signature, add_provider_payload) =
			create_and_sign_add_provider_payload(delegator_pair, provider_msa);

		// Register provider
		assert_ok!(Msa::create_provider(
			RuntimeOrigin::signed(provider_account.into()),
			Vec::from("Foo")
		));

		assert_ok!(Msa::grant_delegation(
			RuntimeOrigin::signed(provider_account.into()),
			delegator_account.into(),
			delegator_signature,
			add_provider_payload
		));

		let delegator_msa =
			Msa::ensure_valid_msa_key(&AccountId32::new(delegator_account.0)).unwrap();

		let provider = ProviderId(provider_msa);
		let delegator = DelegatorId(delegator_msa);

		assert_ok!(Msa::revoke_provider(provider, delegator));

		assert_eq!(
			Msa::get_delegation(delegator, provider).unwrap(),
			Delegation { revoked_at: 1, schema_permissions: Default::default() },
		);
	});
}

#[test]
fn revoke_delegation_by_delegator_does_nothing_when_no_msa() {
	new_test_ext()
		.execute_with(|| assert_revoke_delegation_by_delegator_no_effect(test_public(3), 333u64));
}

#[test]
pub fn revoke_delegation_by_delegator_does_nothing_if_only_key_is_revoked() {
	new_test_ext().execute_with(|| {
		assert_ok!(Msa::create(RuntimeOrigin::signed(test_public(2))));
		assert_ok!(Msa::delete_key_for_msa(1, &test_public(2)));
		assert_revoke_delegation_by_delegator_no_effect(test_public(2), 1u64)
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

		assert_ok!(Msa::create(RuntimeOrigin::signed(delegator_account.into())));
		assert_ok!(Msa::create(RuntimeOrigin::signed(provider_account.into())));

		let provider_msa =
			Msa::ensure_valid_msa_key(&AccountId32::new(provider_account.0)).unwrap();

		let (delegator_signature, add_provider_payload) =
			create_and_sign_add_provider_payload(delegator_pair, provider_msa);

		// Register provider
		assert_ok!(Msa::create_provider(
			RuntimeOrigin::signed(provider_account.into()),
			Vec::from("Foo")
		));

		assert_ok!(Msa::grant_delegation(
			RuntimeOrigin::signed(provider_account.into()),
			delegator_account.into(),
			delegator_signature,
			add_provider_payload
		));

		assert_ok!(Msa::revoke_delegation_by_delegator(
			RuntimeOrigin::signed(delegator_account.into()),
			provider_msa
		));

		assert_noop!(
			Msa::revoke_delegation_by_delegator(
				RuntimeOrigin::signed(delegator_account.into()),
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
		assert_ok!(Msa::create(RuntimeOrigin::signed(provider_account.into())));

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
pub fn delete_msa_public_key_call_has_correct_costs() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let new_key = key_pair.public();

		let call = MsaCall::<Test>::delete_msa_public_key {
			public_key_to_delete: AccountId32::from(new_key),
		};
		let dispatch_info = call.get_dispatch_info();
		assert_eq!(dispatch_info.pays_fee, Pays::No);
	})
}

/// Assert that when a key has been added to an MSA, that it my NOT be added to any other MSA.
/// Expected error: KeyAlreadyRegistered
#[test]
fn double_add_key_two_msa_fails() {
	new_test_ext().execute_with(|| {
		let (msa_id1, owner_key_pair) = create_account();
		let (_msa_id2, msa_2_owner_key_pair) = create_account();

		let add_new_key_data = AddKeyData {
			msa_id: msa_id1,
			expiration: 10,
			new_public_key: msa_2_owner_key_pair.public().into(),
		};
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let owner_signature: MultiSignature = owner_key_pair.sign(&encode_data_new_key_data).into();
		let new_key_signature: MultiSignature =
			msa_2_owner_key_pair.sign(&encode_data_new_key_data).into();

		assert_noop!(
			Msa::add_public_key_to_msa(
				test_origin_signed(1),
				owner_key_pair.public().into(),
				owner_signature,
				new_key_signature,
				add_new_key_data
			),
			Error::<Test>::KeyAlreadyRegistered
		);
	})
}

#[test]
fn add_public_key_to_msa_registers_two_signatures() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let (msa_id1, owner_key_pair) = create_account();
		let (_msa_id2, _msa_2_owner_key_pair) = create_account();
		let (new_key_pair, _) = sr25519::Pair::generate();

		let add_new_key_data = AddKeyData {
			msa_id: msa_id1,
			expiration: 10,
			new_public_key: new_key_pair.public().into(),
		};
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let owner_signature: MultiSignature = owner_key_pair.sign(&encode_data_new_key_data).into();
		let new_key_signature: MultiSignature = new_key_pair.sign(&encode_data_new_key_data).into();

		assert_ok!(Msa::add_public_key_to_msa(
			test_origin_signed(1),
			owner_key_pair.public().into(),
			owner_signature.clone(),
			new_key_signature.clone(),
			add_new_key_data
		));

		assert_eq!(Msa::get_payload_signature_registry(owner_signature.clone()).unwrap().0, 10);
		assert_eq!(
			Msa::get_payload_signature_pointer().unwrap(),
			SignatureRegistryPointer {
				newest: new_key_signature,
				newest_expires_at: 10u32.into(),
				oldest: owner_signature,
				count: 2,
			}
		);
	});
}

/// Assert that when a key has been deleted from one MSA, that it may be added to a different MSA.
#[test]
fn add_removed_key_to_msa_pass() {
	new_test_ext().execute_with(|| {
		let (msa_getting_a_second_key, owner_key_pair) = create_account();
		let (msa_used_to_have_a_key, prior_msa_key) = create_account();

		assert_ok!(Msa::delete_key_for_msa(msa_used_to_have_a_key, &prior_msa_key.public().into()));

		let add_new_key_data = AddKeyData {
			msa_id: msa_getting_a_second_key,
			expiration: 10,
			new_public_key: prior_msa_key.public().into(),
		};
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());
		let owner_signature: MultiSignature = owner_key_pair.sign(&encode_data_new_key_data).into();
		let new_key_signature: MultiSignature =
			prior_msa_key.sign(&encode_data_new_key_data).into();

		assert_ok!(Msa::add_public_key_to_msa(
			test_origin_signed(1),
			owner_key_pair.public().into(),
			owner_signature,
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
			RuntimeOrigin::signed(key_pair.public().into()),
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
				RuntimeOrigin::signed(key_pair.public().into()),
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
			RuntimeOrigin::signed(key_pair.public().into()),
			Vec::from("Foo")
		));

		assert_err!(
			Msa::create_provider(RuntimeOrigin::signed(key_pair.public().into()), Vec::from("Foo")),
			Error::<Test>::DuplicateProviderRegistryEntry
		)
	})
}

pub fn set_schema_count<T: Config>(n: u16) {
	<T>::SchemaValidator::set_schema_count(n);
}

#[test]
pub fn valid_schema_grant() {
	new_test_ext().execute_with(|| {
		set_schema_count::<Test>(2);

		let provider = ProviderId(1);
		let delegator = DelegatorId(2);
		let schema_grants = vec![1, 2];
		assert_ok!(Msa::add_provider(provider, delegator, schema_grants));

		System::set_block_number(System::block_number() + 1);

		assert_ok!(Msa::ensure_valid_schema_grant(provider, delegator, 2u16, 1u64));
	})
}

#[test]
pub fn error_invalid_schema_id() {
	struct TestCase<T> {
		schema: Vec<u16>,
		expected: T,
	}
	new_test_ext().execute_with(|| {
		set_schema_count::<Test>(12);

		let provider = ProviderId(1);
		let delegator = DelegatorId(2);
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

		let provider = ProviderId(1);
		let delegator = DelegatorId(2);
		assert_noop!(
			Msa::add_provider(provider, delegator, (1..32 as u16).collect::<Vec<_>>()),
			Error::<Test>::ExceedsMaxSchemaGrantsPerDelegation
		);
	})
}

#[test]
pub fn error_not_delegated_rpc() {
	new_test_ext().execute_with(|| {
		let provider = ProviderId(1);
		let delegator = DelegatorId(2);
		assert_err!(
			Msa::get_granted_schemas_by_msa_id(delegator, provider),
			Error::<Test>::DelegationNotFound
		);
	})
}

#[test]
pub fn error_schema_not_granted_rpc() {
	new_test_ext().execute_with(|| {
		let provider = ProviderId(1);
		let delegator = DelegatorId(2);
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

		let provider = ProviderId(1);
		let delegator = DelegatorId(2);
		let schema_grants = vec![1, 2];
		assert_ok!(Msa::add_provider(provider, delegator, schema_grants));
		let schemas_granted = Msa::get_granted_schemas_by_msa_id(delegator, provider);
		let expected_schemas_granted = vec![1, 2];
		let output_schemas: Vec<SchemaId> = schemas_granted.unwrap().unwrap();
		assert_eq!(output_schemas, expected_schemas_granted);
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
		assert_ok!(Msa::create(RuntimeOrigin::signed(provider_key.into()))); // MSA = 1

		// Register provider
		assert_ok!(Msa::create_provider(
			RuntimeOrigin::signed(provider_key.into()),
			Vec::from("Foo")
		));

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
		let provider = ProviderId(1);
		let provider_name = Vec::from("frequency".as_bytes()).try_into().unwrap();

		let provider_meta = ProviderRegistryEntry { provider_name };
		ProviderToRegistryEntry::<Test>::insert(provider, provider_meta);

		assert!(Msa::is_registered_provider(provider.into()));
	});
}

#[test]
fn grant_permissions_for_schemas_errors_when_no_delegation() {
	new_test_ext().execute_with(|| {
		let delegator = DelegatorId(2);
		let provider = ProviderId(1);
		let schema_ids = vec![1, 2];
		let result = Msa::grant_permissions_for_schemas(delegator, provider, schema_ids);

		assert_noop!(result, Error::<Test>::DelegationNotFound);
	});
}

#[test]
fn grant_permissions_for_schemas_errors_when_invalid_schema_id() {
	new_test_ext().execute_with(|| {
		set_schema_count::<Test>(1);
		let delegator = DelegatorId(2);
		let provider = ProviderId(1);
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

		let delegator = DelegatorId(2);
		let provider = ProviderId(1);
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

		let delegator = DelegatorId(2);
		let provider = ProviderId(1);
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
		let delegator = DelegatorId(1);
		let provider = ProviderId(2);

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

		let delegator = DelegatorId(2);
		let provider = ProviderId(1);
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
		let delegator = DelegatorId(2);
		let provider = ProviderId(1);
		let schema_ids = vec![1, 2];
		let result = Msa::revoke_permissions_for_schemas(delegator, provider, schema_ids);

		assert_noop!(result, Error::<Test>::DelegationNotFound);
	});
}

#[test]
fn revoke_permissions_for_schemas_errors_when_schema_does_not_exist_in_list_of_schema_grants() {
	new_test_ext().execute_with(|| {
		set_schema_count::<Test>(31);

		let delegator = DelegatorId(2);
		let provider = ProviderId(1);
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

		let provider = ProviderId(1);
		let delegator = DelegatorId(2);
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

		let provider = ProviderId(1);
		let delegator = DelegatorId(2);
		let schema_grants = vec![1, 2];

		// Add delegation relationship with schema grants.
		assert_ok!(Msa::add_provider(provider, delegator, schema_grants));

		// Set block number to 2.
		System::set_block_number(System::block_number() + 1);

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

		// Create delegation relationship.
		let provider = ProviderId(1);
		let delegator = DelegatorId(2);
		let schema_grants = vec![1, 2];
		assert_ok!(Msa::add_provider(provider, delegator, schema_grants));

		// Set block number to 6.
		System::set_block_number(System::block_number() + 5);

		// revoke schema permission at block 6.
		assert_ok!(Msa::revoke_permissions_for_schemas(delegator, provider, vec![1]));

		// Schemas is valid for the current block that is revoked 6
		assert_ok!(Msa::ensure_valid_schema_grant(provider, delegator, 1, 6));

		// Checking that asking for validity past the current block, 6, errors.
		assert_noop!(
			Msa::ensure_valid_schema_grant(provider, delegator, 1, 7),
			Error::<Test>::CannotPredictValidityPastCurrentBlock
		);

		// Set block number to 6.
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
		// Set the schemas counts so that it passes validation.
		set_schema_count::<Test>(2);

		// Create delegation relationship.
		let provider = ProviderId(1);
		let delegator = DelegatorId(2);
		let schema_grants = vec![1, 2];

		// Create delegation relationship.
		assert_ok!(Msa::add_provider(provider, delegator, schema_grants));

		// Move forward to block 6.
		System::set_block_number(System::block_number() + 5);

		// Revoke delegation relationship at block 6.
		assert_ok!(Msa::revoke_provider(provider, delegator));

		// Schemas is valid for the current block that is revoked 6.
		assert_ok!(Msa::ensure_valid_schema_grant(provider, delegator, 1, 6));
		assert_ok!(Msa::ensure_valid_schema_grant(provider, delegator, 1, 5));

		// Checking that asking for validity past the current block, 6, errors.
		assert_noop!(
			Msa::ensure_valid_schema_grant(provider, delegator, 1, 8),
			Error::<Test>::CannotPredictValidityPastCurrentBlock
		);

		// Move forward to block 11.
		System::set_block_number(System::block_number() + 5);

		// Check that schema is not valid after delegation revocation
		assert_noop!(
			Msa::ensure_valid_schema_grant(provider, delegator, 1, 7),
			Error::<Test>::DelegationRevoked
		);
	});
}
