use crate::{
	ensure,
	mock::*,
	types::{
		AccountCreationDelegate, AddDelegate, AddKeyData, Delegate, DelegateInfo, Delegator,
		KeyInfo, EMPTY_FUNCTION,
	},
	Call, Config, DispatchResult, Error, Event, MsaIdentifier,
};
use common_primitives::{msa::KeyInfoResponse, utils::wrap_binary_data};
use frame_support::{assert_noop, assert_ok, weights::GetDispatchInfo};
use sp_core::{crypto::AccountId32, sr25519, Encode, Pair};
use sp_runtime::MultiSignature;

#[test]
fn it_creates_an_msa_account() {
	new_test_ext().execute_with(|| {
		assert_ok!(Msa::create(test_origin_signed(1)));

		assert_eq!(
			Msa::get_key_info(test_public(1)),
			Some(KeyInfo { msa_id: 1, expired: 0, nonce: 0 })
		);

		assert_eq!(Msa::get_identifier(), 1);

		System::assert_last_event(
			Event::MsaCreated { msa_id: 1, key: test_public(1).into() }.into(),
		);
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

		assert_noop!(Msa::create(test_origin_signed(1)), Error::<Test>::DuplicatedKey);

		assert_eq!(Msa::get_identifier(), 1);
	});
}

#[test]
fn it_create_has_weight() {
	new_test_ext().execute_with(|| {
		let call = Call::<Test>::create {};
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
				add_new_key_data.clone()
			),
			Error::<Test>::KeyVerificationFailed
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
		assert_ok!(Msa::create_account(test_public(1).into(), EMPTY_FUNCTION));

		let new_account = key_pair_2.public();

		let add_new_key_data = AddKeyData { nonce: 0, msa_id: new_msa_id };
		let encode_data_new_key_data = wrap_binary_data(add_new_key_data.encode());

		let signature: MultiSignature = key_pair_2.sign(&encode_data_new_key_data).into();

		assert_noop!(
			Msa::add_key_to_msa(
				test_origin_signed(1),
				new_account.into(),
				signature,
				add_new_key_data.clone()
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
				add_new_key_data.clone()
			),
			Error::<Test>::DuplicatedKey
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
				add_new_key_data.clone()
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
			add_new_key_data.clone(),
		));

		// assert
		let keys = Msa::fetch_msa_keys(new_msa_id);
		assert_eq!(keys.len(), 2);
		assert_eq!{keys.contains(&KeyInfoResponse {key: AccountId32::from(new_key), msa_id: new_msa_id, nonce: 0, expired: 0}), true}
		System::assert_last_event(Event::KeyAdded { msa_id: 1, key: new_key.into() }.into());
	});
}

#[test]
fn it_revokes_msa_key_successfully() {
	new_test_ext().execute_with(|| {
		assert_ok!(Msa::add_key(2, &test_public(1), EMPTY_FUNCTION));
		assert_ok!(Msa::add_key(2, &test_public(2), EMPTY_FUNCTION));

		assert_ok!(Msa::revoke_msa_key(test_origin_signed(1), test_public(2)));

		let info = Msa::get_key_info(&test_public(2));

		assert_eq!(info, Some(KeyInfo { msa_id: 2, expired: 1, nonce: 0 }));

		System::assert_last_event(Event::KeyRevoked { key: test_public(2).into() }.into());
	})
}

#[test]
pub fn test_get_onwner_of() {
	new_test_ext().execute_with(|| {
		assert_eq!(Msa::get_owner_of(&test_public(1)), None);

		assert_ok!(Msa::create(test_origin_signed(1)));

		assert_eq!(Msa::get_owner_of(&test_public(1)), Some(1));
	});
}

#[test]
pub fn test_revoke_key() {
	new_test_ext().execute_with(|| {
		assert_ok!(Msa::add_key(1, &test_public(1), EMPTY_FUNCTION));

		let info = Msa::get_key_info(&test_public(1));
		assert_eq!(info, Some(KeyInfo { msa_id: 1, expired: 0, nonce: 0 }));

		assert_ok!(Msa::revoke_key(&test_public(1)));

		let info = Msa::get_key_info(&test_public(1));

		assert_eq!(info, Some(KeyInfo { msa_id: 1, expired: 1, nonce: 0 }));
	});
}

#[test]
pub fn test_revoke_key_errors() {
	new_test_ext().execute_with(|| {
		assert_noop!(Msa::revoke_key(&test_public(1)), Error::<Test>::NoKeyExists);

		assert_ok!(Msa::add_key(1, &test_public(1), EMPTY_FUNCTION));
		assert_ok!(Msa::revoke_key(&test_public(1)));

		assert_noop!(Msa::revoke_key(&test_public(1)), Error::<Test>::KeyRevoked);
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
pub fn add_delegate_to_msa_is_success() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let delegate_account = key_pair.public();

		let add_delegate_payload = AddDelegate { authorized_msa_id: 1, permission: 0 };
		let encode_add_delegate_data = wrap_binary_data(add_delegate_payload.encode());

		let signature: MultiSignature = key_pair.sign(&encode_add_delegate_data).into();

		assert_ok!(Msa::create(test_origin_signed(1)));
		assert_ok!(Msa::create(Origin::signed(delegate_account.into())));

		assert_ok!(Msa::add_delegate_to_msa(
			test_origin_signed(1),
			delegate_account.into(),
			signature,
			add_delegate_payload
		));

		let delegate = Delegate(2);
		let delegator = Delegator(1);

		assert_eq!(
			Msa::get_delegate_info_of(delegate, delegator),
			Some(DelegateInfo { permission: 0, expired: 0 })
		);

		System::assert_last_event(
			Event::DelegateAdded { delegator: 1.into(), delegate: 2.into() }.into(),
		);
	});
}

#[test]
pub fn add_delegate_to_msa_throws_add_delegate_verification_failed() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let account = key_pair.public();

		let add_delegate_payload = AddDelegate { authorized_msa_id: 2, permission: 0 };
		let encode_add_delegate_data = wrap_binary_data(add_delegate_payload.encode());

		let signature: MultiSignature = key_pair.sign(&encode_add_delegate_data).into();

		let fake_delegate_payload = AddDelegate { authorized_msa_id: 3, permission: 0 };

		assert_noop!(
			Msa::add_delegate_to_msa(
				test_origin_signed(1),
				account.into(),
				signature,
				fake_delegate_payload
			),
			Error::<Test>::AddDelegateVerificationFailed
		);
	});
}

#[test]
pub fn add_delegate_to_msa_throws_no_key_exist_error() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let delegate_account = key_pair.public();

		let add_delegate_payload = AddDelegate { authorized_msa_id: 2, permission: 0 };
		let encode_add_delegate_data = wrap_binary_data(add_delegate_payload.encode());

		let signature: MultiSignature = key_pair.sign(&encode_add_delegate_data).into();

		assert_noop!(
			Msa::add_delegate_to_msa(
				test_origin_signed(1),
				delegate_account.into(),
				signature,
				add_delegate_payload
			),
			Error::<Test>::NoKeyExists
		);
	});
}

#[test]
pub fn add_delegate_to_msa_throws_key_revoked_error() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let delegate_account = key_pair.public();

		let add_delegate_payload = AddDelegate { authorized_msa_id: 2, permission: 0 };
		let encode_add_delegate_data = wrap_binary_data(add_delegate_payload.encode());

		let signature: MultiSignature = key_pair.sign(&encode_add_delegate_data).into();

		assert_ok!(Msa::create(test_origin_signed(1)));
		assert_ok!(Msa::create(Origin::signed(delegate_account.into())));
		assert_ok!(Msa::revoke_key(&test_public(1)));

		assert_noop!(
			Msa::add_delegate_to_msa(
				test_origin_signed(1),
				delegate_account.into(),
				signature,
				add_delegate_payload
			),
			Error::<Test>::KeyRevoked
		);
	});
}

#[test]
pub fn add_delegate_to_msa_throws_invalid_self_delegate_error() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let delegate_account = key_pair.public();

		let add_delegate_payload = AddDelegate { authorized_msa_id: 1, permission: 0 };

		let encode_add_delegate_data = wrap_binary_data(add_delegate_payload.encode());

		let signature: MultiSignature = key_pair.sign(&encode_add_delegate_data).into();

		assert_ok!(Msa::create(Origin::signed(delegate_account.into())));

		assert_noop!(
			Msa::add_delegate_to_msa(
				Origin::signed(delegate_account.into()),
				delegate_account.into(),
				signature,
				add_delegate_payload
			),
			Error::<Test>::InvalidSelfDelegate
		);
	});
}

#[test]
pub fn add_delegate_to_msa_throws_unauthorized_delegator_error() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let delegate_account = key_pair.public();

		let add_delegate_payload = AddDelegate { authorized_msa_id: 2, permission: 0 };

		let encode_add_delegate_data = wrap_binary_data(add_delegate_payload.encode());

		let signature: MultiSignature = key_pair.sign(&encode_add_delegate_data).into();

		assert_ok!(Msa::create(Origin::signed(delegate_account.into())));

		assert_noop!(
			Msa::add_delegate_to_msa(
				Origin::signed(delegate_account.into()),
				delegate_account.into(),
				signature,
				add_delegate_payload
			),
			Error::<Test>::UnauthorizedDelegator
		);
	});
}

#[test]
pub fn add_delegate_to_msa_throws_duplicate_delegate_error() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let delegate_account = key_pair.public();

		let add_delegate_payload = AddDelegate { authorized_msa_id: 1, permission: 0 };
		let encode_add_delegate_data = wrap_binary_data(add_delegate_payload.encode());

		let signature: MultiSignature = key_pair.sign(&encode_add_delegate_data).into();

		assert_ok!(Msa::create(test_origin_signed(1)));
		assert_ok!(Msa::create(Origin::signed(delegate_account.into())));

		assert_ok!(Msa::add_delegate_to_msa(
			test_origin_signed(1),
			delegate_account.into(),
			signature.clone(),
			add_delegate_payload.clone()
		));

		assert_noop!(
			Msa::add_delegate_to_msa(
				test_origin_signed(1),
				delegate_account.into(),
				signature,
				add_delegate_payload
			),
			Error::<Test>::DuplicateDelegate
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
pub fn create_account_with_delegate_with_valid_input_should_succeed() {
	new_test_ext().execute_with(|| {
		// arrange
		let (key_pair, _) = sr25519::Pair::generate();
		let delegate_account = key_pair.public();

		let (key_pair_delegator, _) = sr25519::Pair::generate();
		let delegator_account = key_pair_delegator.public();

		let account_creation_delegate_payload =
			AccountCreationDelegate { key: delegator_account.into(), permission: 0 };
		let encode_account_creation_delegate_data =
			wrap_binary_data(account_creation_delegate_payload.encode());

		let signature: MultiSignature =
			key_pair_delegator.sign(&encode_account_creation_delegate_data).into();

		assert_ok!(Msa::create(Origin::signed(delegate_account.into())));

		// act
		assert_ok!(Msa::create_account_with_delegate(
			Origin::signed(delegate_account.into()),
			signature.clone(),
			account_creation_delegate_payload.clone()
		));

		// assert
		let key_info = Msa::get_key_info(AccountId32::new(delegator_account.0));
		assert_eq!(key_info.unwrap().msa_id, 2);

		let delegated = Msa::get_delegate_info_of(Delegate(1), Delegator(2));
		assert_eq!(delegated.is_some(), true);

		System::assert_last_event(
			Event::MsaCreatedAndDelegated {
				new_msa_id: 2u64.into(),
				delegator_key: delegator_account.into(),
				delegate: 1u64.into(),
			}
			.into(),
		);
	});
}

#[test]
fn create_account_with_delegate_with_invalid_signature_should_fail() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let delegate_account = key_pair.public();

		let (key_pair_delegator, _) = sr25519::Pair::generate();
		let delegator_account = key_pair_delegator.public();

		let (signer_pair, _) = sr25519::Pair::generate();

		let account_creation_delegate_payload =
			AccountCreationDelegate { key: delegator_account.into(), permission: 0 };
		let encode_account_creation_delegate_data =
			wrap_binary_data(account_creation_delegate_payload.encode());

		let signature: MultiSignature =
			signer_pair.sign(&encode_account_creation_delegate_data).into();

		assert_ok!(Msa::create(Origin::signed(delegate_account.into())));

		// act
		assert_noop!(
			Msa::create_account_with_delegate(
				Origin::signed(delegate_account.into()),
				signature.clone(),
				account_creation_delegate_payload.clone()
			),
			Error::<Test>::InvalidSignature
		);
	});
}

#[test]
pub fn create_account_with_delegate_with_invalid_creation_delegate_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let (key_pair, _) = sr25519::Pair::generate();
		let delegate_account = key_pair.public();

		let (key_pair_delegator, _) = sr25519::Pair::generate();
		let delegator_account = key_pair_delegator.public();

		let account_creation_delegate_payload =
			AccountCreationDelegate { key: delegator_account.into(), permission: 0 };
		let encode_account_creation_delegate_data =
			wrap_binary_data(account_creation_delegate_payload.encode());

		let signature: MultiSignature =
			key_pair_delegator.sign(&encode_account_creation_delegate_data).into();

		assert_ok!(Msa::create(Origin::signed(delegate_account.into())));
		assert_ok!(Msa::create(Origin::signed(delegator_account.into())));

		// act
		assert_noop!(
			Msa::create_account_with_delegate(
				Origin::signed(delegate_account.into()),
				signature.clone(),
				account_creation_delegate_payload.clone()
			),
			Error::<Test>::AlreadyAssignedKey
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
				ensure!(new_msa_id != msa_id, Error::<Test>::InvalidSelfRevoke);
				Ok(())
			}),
			Error::<Test>::InvalidSelfRevoke
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
				ensure!(new_msa_id != msa_id, Error::<Test>::InvalidSelfRevoke);
				Ok(())
			}),
			Error::<Test>::InvalidSelfRevoke
		);

		// assert
		assert_eq!(next_msa_id, Msa::get_next_msa_id().unwrap());
	});
}

#[test]
pub fn revoke_msa_delegate_is_successfull() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let delegate_account = key_pair.public();

		let add_delegate_payload = AddDelegate { authorized_msa_id: 1, permission: 0 };
		let encode_add_delegate_data = wrap_binary_data(add_delegate_payload.encode());

		let signature: MultiSignature = key_pair.sign(&encode_add_delegate_data).into();

		assert_ok!(Msa::create(test_origin_signed(1)));
		assert_ok!(Msa::create(Origin::signed(delegate_account.into())));

		assert_ok!(Msa::add_delegate_to_msa(
			test_origin_signed(1),
			delegate_account.into(),
			signature,
			add_delegate_payload
		));

		assert_ok!(Msa::revoke_msa_delegate(test_origin_signed(1), 2));

		System::assert_last_event(
			Event::DelegateRevoked { delegator: 1.into(), delegate: 2.into() }.into(),
		);
	});
}

#[test]
pub fn revoke_delegate_is_successfull() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();
		let delegate_account = key_pair.public();

		let add_delegate_payload = AddDelegate { authorized_msa_id: 1, permission: 0 };
		let encode_add_delegate_data = wrap_binary_data(add_delegate_payload.encode());

		let signature: MultiSignature = key_pair.sign(&encode_add_delegate_data).into();

		assert_ok!(Msa::create(test_origin_signed(1)));
		assert_ok!(Msa::create(Origin::signed(delegate_account.into())));
		assert_ok!(Msa::add_delegate_to_msa(
			test_origin_signed(1),
			delegate_account.into(),
			signature.clone(),
			add_delegate_payload.clone()
		));

		let delegate = Delegate(2);
		let delegator = Delegator(1);

		assert_ok!(Msa::revoke_delegate(delegate, delegator));

		assert_eq!(
			Msa::get_delegate_info_of(delegate, delegator).unwrap(),
			DelegateInfo { expired: 1, permission: 0 },
		);
	});
}

#[test]
fn revoke_delegate_throws_errors() {
	new_test_ext().execute_with(|| {
		let (key_pair, _) = sr25519::Pair::generate();

		let delegate_account = key_pair.public();
		let add_delegate_payload = AddDelegate { authorized_msa_id: 2, permission: 0 };
		let encode_add_delegate_data = wrap_binary_data(add_delegate_payload.encode());

		let signature: MultiSignature = key_pair.sign(&encode_add_delegate_data).into();

		assert_noop!(
			Msa::revoke_msa_delegate(test_origin_signed(1), 1),
			Error::<Test>::NoKeyExists
		);

		assert_ok!(Msa::create(test_origin_signed(2)));
		assert_ok!(Msa::revoke_key(&test_public(2)));
		assert_noop!(Msa::revoke_msa_delegate(test_origin_signed(2), 1), Error::<Test>::KeyRevoked);

		assert_ok!(Msa::create(test_origin_signed(1)));
		assert_noop!(
			Msa::revoke_msa_delegate(test_origin_signed(1), 4),
			Error::<Test>::DelegateNotFound
		);

		assert_ok!(Msa::create(Origin::signed(delegate_account.into())));

		assert_ok!(Msa::add_delegate_to_msa(
			test_origin_signed(1),
			delegate_account.into(),
			signature.clone(),
			add_delegate_payload.clone()
		));

		assert_noop!(
			Msa::revoke_msa_delegate(test_origin_signed(1), 4),
			Error::<Test>::DelegateNotFound
		);

		assert_ok!(Msa::revoke_msa_delegate(test_origin_signed(1), 3));

		assert_noop!(
			Msa::revoke_msa_delegate(test_origin_signed(1), 3),
			Error::<Test>::DelegateRevoked
		);
	});
}

#[test]
pub fn revoke_delegate_throws_delegate_not_found_error() {
	new_test_ext().execute_with(|| {
		let delegate = Delegate(1);
		let delegator = Delegator(2);

		assert_noop!(Msa::revoke_delegate(delegate, delegator), Error::<Test>::DelegateNotFound);
	});
}
