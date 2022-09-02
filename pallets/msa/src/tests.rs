use crate::{
	ensure,
	mock::*,
	types::{AddKeyData, AddProvider, EMPTY_FUNCTION},
	CheckFreeExtrinsicUse, Config, DispatchResult, Error, Event, MsaIdentifier,
};
use common_primitives::{
	msa::{Delegator, KeyInfoResponse, Provider, ProviderInfo, MessageSourceId},
	utils::wrap_binary_data,
};
use frame_support::{
	assert_err, assert_noop, assert_ok,
	weights::{DispatchInfo, GetDispatchInfo, Pays},
};
use sp_core::{crypto::AccountId32, sr25519, Encode, Pair};
use sp_runtime::{traits::SignedExtension, MultiSignature};

#[test]
fn it_creates_an_msa_account() {
	new_test_ext().execute_with(|| {
		assert_ok!(Msa::create(test_origin_signed(1)));

		assert_eq!(Msa::get_key_info(test_public(1)), Some(1 as MessageSourceId));

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

		assert!(dispatch_info.weight > 10_000);
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

		let add_new_key_data = AddKeyData { nonce: 1, msa_id: new_msa_id };
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

		let add_new_key_data = AddKeyData { nonce: 0, msa_id: new_msa_id };
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

		let add_new_key_data = AddKeyData { nonce: 1, msa_id: new_msa_id };
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
		let add_new_key_data = AddKeyData { nonce: 1, msa_id: new_msa_id };
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

		let add_new_key_data = AddKeyData { nonce: 1, msa_id: new_msa_id };
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
		let keys = Msa::fetch_msa_keys(new_msa_id);
		assert_eq!(keys.len(), 2);
		assert_eq!{keys.contains(&KeyInfoResponse {key: AccountId32::from(new_key), msa_id: new_msa_id, nonce: 0}), true}
		System::assert_last_event(Event::KeyAdded { msa_id: 1, key: new_key.into() }.into());
	});
}

#[test]
fn it_revokes_msa_key_successfully() {
	new_test_ext().execute_with(|| {
		assert_ok!(Msa::add_key(2, &test_public(1), EMPTY_FUNCTION));
		assert_ok!(Msa::add_key(2, &test_public(2), EMPTY_FUNCTION));

		assert_ok!(Msa::delete_msa_key(test_origin_signed(1), test_public(2)));

		let info = Msa::get_key_info(&test_public(2));

		assert_eq!(info, None);

		System::assert_last_event(Event::KeyRemoved { key: test_public(2) }.into());
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

		let info = Msa::get_key_info(&test_public(1));

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

		let add_provider_payload = AddProvider { authorized_msa_id: 1, permission: 0 };
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = key_pair.sign(&encode_add_provider_data).into();

		assert_ok!(Msa::create(test_origin_signed(1)));
		assert_ok!(Msa::create(Origin::signed(provider_account.into())));

		assert_ok!(Msa::add_provider_to_msa(
			test_origin_signed(1),
			provider_account.into(),
			signature,
			add_provider_payload
		));

		let provider = Provider(2);
		let delegator = Delegator(1);

		assert_eq!(
			Msa::get_provider_info_of(delegator, provider),
			Some(ProviderInfo { permission: 0, expired: 0 })
		);

		System::assert_last_event(
			Event::ProviderAdded { delegator: 1.into(), provider: 2.into() }.into(),
		);
	});
}

#[test]
pub fn add_provider_to_msa_throws_add_provider_verification_failed() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let account = key_pair.public();

		let add_provider_payload = AddProvider { authorized_msa_id: 2, permission: 0 };
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = key_pair.sign(&encode_add_provider_data).into();

		let fake_provider_payload = AddProvider { authorized_msa_id: 3, permission: 0 };

		assert_noop!(
			Msa::add_provider_to_msa(
				test_origin_signed(1),
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

		let add_provider_payload = AddProvider { authorized_msa_id: 2, permission: 0 };
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

		let add_provider_payload = AddProvider { authorized_msa_id: 2, permission: 0 };
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

		let add_provider_payload = AddProvider { authorized_msa_id: 1, permission: 0 };

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
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let add_provider_payload = AddProvider { authorized_msa_id: 2, permission: 0 };

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
			Error::<Test>::UnauthorizedDelegator
		);
	});
}

#[test]
pub fn add_provider_to_msa_throws_duplicate_provider_error() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let add_provider_payload = AddProvider { authorized_msa_id: 1, permission: 0 };
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = key_pair.sign(&encode_add_provider_data).into();

		assert_ok!(Msa::create(test_origin_signed(1)));
		assert_ok!(Msa::create(Origin::signed(provider_account.into())));

		assert_ok!(Msa::add_provider_to_msa(
			test_origin_signed(1),
			provider_account.into(),
			signature.clone(),
			add_provider_payload.clone()
		));

		assert_noop!(
			Msa::add_provider_to_msa(
				test_origin_signed(1),
				provider_account.into(),
				signature,
				add_provider_payload
			),
			Error::<Test>::DuplicateProvider
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

		let add_provider_payload = AddProvider { authorized_msa_id: 1u64, permission: 0 };
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = key_pair_delegator.sign(&encode_add_provider_data).into();

		assert_ok!(Msa::create(Origin::signed(provider_account.into())));

		// act
		assert_ok!(Msa::create_sponsored_account_with_delegation(
			Origin::signed(provider_account.into()),
			delegator_account.into(),
			signature,
			add_provider_payload
		));

		// assert
		let key_info = Msa::get_key_info(AccountId32::new(delegator_account.0));
		assert_eq!(key_info.unwrap(), 2);

		let provider_info = Msa::get_provider_info_of(Delegator(2), Provider(1));
		assert_eq!(provider_info.is_some(), true);

		let events_occured = System::events();
		let created_event = &events_occured.as_slice()[1];
		let provider_event = &events_occured.as_slice()[2];
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

		let add_provider_payload = AddProvider { authorized_msa_id: 1u64, permission: 0 };
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

		let add_provider_payload = AddProvider { authorized_msa_id: 1u64, permission: 0 };
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = key_pair_delegator.sign(&encode_add_provider_data).into();

		assert_ok!(Msa::create(Origin::signed(provider_account.into())));
		assert_ok!(Msa::create(Origin::signed(delegator_account.into())));

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

		let add_provider_payload = AddProvider { authorized_msa_id: 3u64, permission: 0 };
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
		assert_eq!(Msa::get_key_info(&key), None);

		assert_eq!(Msa::get_msa_keys(msa_id).into_inner(), vec![])
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
pub fn revoke_msa_delegation_by_delegator_is_successfull() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let provider_account = key_pair.public();

		let add_provider_payload = AddProvider { authorized_msa_id: 1, permission: 0 };
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = key_pair.sign(&encode_add_provider_data).into();

		assert_ok!(Msa::create(test_origin_signed(1)));
		assert_ok!(Msa::create(Origin::signed(provider_account.into())));

		assert_ok!(Msa::add_provider_to_msa(
			test_origin_signed(1),
			provider_account.into(),
			signature,
			add_provider_payload
		));

		assert_ok!(Msa::revoke_msa_delegation_by_delegator(test_origin_signed(1), 2));

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

		let add_provider_payload = AddProvider { authorized_msa_id: 1, permission: 0 };
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = key_pair.sign(&encode_add_provider_data).into();

		assert_ok!(Msa::create(test_origin_signed(1)));
		assert_ok!(Msa::create(Origin::signed(provider_account.into())));
		assert_ok!(Msa::add_provider_to_msa(
			test_origin_signed(1),
			provider_account.into(),
			signature,
			add_provider_payload
		));

		let provider = Provider(2);
		let delegator = Delegator(1);

		assert_ok!(Msa::revoke_provider(provider, delegator));

		assert_eq!(
			Msa::get_provider_info_of(delegator, provider).unwrap(),
			ProviderInfo { expired: 1, permission: 0 },
		);
	});
}

#[test]
fn revoke_provider_throws_errors() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();

		let provider_account = key_pair.public();
		let add_provider_payload = AddProvider { authorized_msa_id: 2, permission: 0 };
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = key_pair.sign(&encode_add_provider_data).into();

		assert_noop!(
			Msa::revoke_msa_delegation_by_delegator(test_origin_signed(1), 1),
			Error::<Test>::NoKeyExists
		);

		assert_ok!(Msa::create(test_origin_signed(2)));
		assert_ok!(Msa::delete_key_for_msa(1, &test_public(2)));
		assert_noop!(
			Msa::revoke_msa_delegation_by_delegator(test_origin_signed(2), 1),
			Error::<Test>::NoKeyExists
		);

		assert_ok!(Msa::create(test_origin_signed(1)));
		assert_noop!(
			Msa::revoke_msa_delegation_by_delegator(test_origin_signed(1), 4),
			Error::<Test>::DelegationNotFound
		);

		assert_ok!(Msa::create(Origin::signed(provider_account.into())));

		assert_ok!(Msa::add_provider_to_msa(
			test_origin_signed(1),
			provider_account.into(),
			signature,
			add_provider_payload
		));

		assert_noop!(
			Msa::revoke_msa_delegation_by_delegator(test_origin_signed(1), 4),
			Error::<Test>::DelegationNotFound
		);

		assert_ok!(Msa::revoke_msa_delegation_by_delegator(test_origin_signed(1), 3));

		assert_noop!(
			Msa::revoke_msa_delegation_by_delegator(test_origin_signed(1), 3),
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

		let add_provider_payload = AddProvider { authorized_msa_id: 1, permission: 0 };
		let encode_add_provider_data = wrap_binary_data(add_provider_payload.encode());

		let signature: MultiSignature = key_pair.sign(&encode_add_provider_data).into();

		assert_ok!(Msa::create(test_origin_signed(1)));
		assert_ok!(Msa::create(Origin::signed(provider_account.into())));
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
		assert_ok!(Msa::create(Origin::signed(provider_key.into())));

		// 3. create delegator MSA and provider to provider
		let add_provider_payload = AddProvider { authorized_msa_id: 1u64, permission: 0 };
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
		let provider_info = Msa::get_provider_info_of(Delegator(2), Provider(1));
		assert_eq!(provider_info, Some(ProviderInfo { permission: 0, expired: 26 }));

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

		assert_ok!(Msa::add_provider(Provider(1), Delegator(2)));
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

		assert_ok!(Msa::add_provider(provider, delegator));

		System::set_block_number(System::block_number() + 1);

		assert_ok!(Msa::ensure_valid_delegation(provider, delegator));
	})
}

#[test]
pub fn delegation_not_found() {
	new_test_ext().execute_with(|| {
		let provider = Provider(1);
		let delegator = Delegator(2);

		assert_noop!(
			Msa::ensure_valid_delegation(provider, delegator),
			Error::<Test>::DelegationNotFound
		);
	})
}

#[test]
pub fn delegation_expired() {
	new_test_ext().execute_with(|| {
		let provider = Provider(1);
		let delegator = Delegator(2);

		assert_ok!(Msa::add_provider(provider, delegator));

		System::set_block_number(System::block_number() + 1);
		assert_ok!(Msa::ensure_valid_delegation(provider, delegator));

		assert_ok!(Msa::revoke_provider(provider, delegator));

		System::set_block_number(System::block_number() + 1);

		assert_noop!(
			Msa::ensure_valid_delegation(provider, delegator),
			Error::<Test>::DelegationExpired
		);
	})
}

/// Assert that revoking an MSA delegation passes the signed extension CheckFreeExtrinsicUse
/// validation when a valid delegation exists.
#[test]
fn signed_extension_revoke_msa_delegation_by_delegator() {
	new_test_ext().execute_with(|| {
		let (_, provider_msa_id) = test_create_delegator_msa_with_provider();
		let call_revoke_delegation: &<Test as frame_system::Config>::Call =
			&Call::Msa(MsaCall::revoke_msa_delegation_by_delegator { provider_msa_id });
		let info = DispatchInfo::default();
		let len = 0_usize;
		let result = CheckFreeExtrinsicUse::<Test>::new().validate(
			&test_public(1),
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
		let (_, provider_msa_id) = test_create_delegator_msa_with_provider();
		let call_revoke_delegation: &<Test as frame_system::Config>::Call =
			&Call::Msa(MsaCall::revoke_msa_delegation_by_delegator { provider_msa_id });
		let info = DispatchInfo::default();
		let len = 0_usize;
		let result = CheckFreeExtrinsicUse::<Test>::new().validate(
			&test_public(1),
			call_revoke_delegation,
			&info,
			len,
		);
		assert_ok!(result);
		assert_ok!(Msa::revoke_msa_delegation_by_delegator(test_origin_signed(1), provider_msa_id));
		System::set_block_number(System::block_number() + 1);
		let call_revoke_delegation: &<Test as frame_system::Config>::Call =
			&Call::Msa(MsaCall::revoke_msa_delegation_by_delegator { provider_msa_id });
		let info = DispatchInfo::default();
		let len = 0_usize;
		let result_revoked = CheckFreeExtrinsicUse::<Test>::new().validate(
			&test_public(1),
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

		let add_new_key_data = AddKeyData { nonce: 1, msa_id: msa_id2 };
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

		let add_new_key_data = AddKeyData { nonce: 1, msa_id: msa_id2 };
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
