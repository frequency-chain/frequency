//! Unit tests for the passkey module.
use super::*;
use crate::mock::Passkey;
use frame_support::{assert_err, assert_noop, assert_ok};
use frame_system::Call as SystemCall;
use mock::*;
use pallet_balances::Call as BalancesCall;
use sp_core::{sr25519, Pair};
use sp_runtime::{DispatchError::BadOrigin, MultiSignature};

#[test]
fn proxy_call_with_signed_origin_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let (test_account_1_key_pair, _) = sr25519::Pair::generate();
		let (test_account_2_key_pair, _) = sr25519::Pair::generate();

		let signature: MultiSignature = test_account_1_key_pair.sign(b"sdsds").into();
		let call: PasskeyCall<Test> = PasskeyCall {
			account_id: test_account_1_key_pair.public().into(),
			account_nonce: 3,
			account_ownership_proof: signature,
			call: Box::new(RuntimeCall::Balances(BalancesCall::transfer_allow_death {
				dest: test_account_2_key_pair.public().into(),
				value: 100,
			})),
		};
		let payload = PasskeyPayload {
			passkey_public_key: [0u8; 33],
			verifiable_passkey_signature: VerifiablePasskeySignature {
				signature: PasskeySignature::default(),
				client_data_json: PasskeyClientDataJson::default(),
				authenticator_data: PasskeyAuthenticatorData::default(),
			},
			passkey_call: call,
		};

		// assert
		assert_noop!(
			Passkey::proxy(RuntimeOrigin::signed(test_account_1_key_pair.public().into()), payload),
			BadOrigin
		);
	});
}

#[test]
fn proxy_call_with_unsigned_origin_should_work() {
	new_test_ext().execute_with(|| {
		// arrange
		let (test_account_1_key_pair, _) = sr25519::Pair::generate();
		let signature: MultiSignature = test_account_1_key_pair.sign(b"sdsds").into();
		let call: PasskeyCall<Test> = PasskeyCall {
			account_id: test_account_1_key_pair.public().into(),
			account_nonce: 3,
			account_ownership_proof: signature,
			call: Box::new(RuntimeCall::System(SystemCall::remark { remark: vec![1, 2, 3u8] })),
		};
		let payload = PasskeyPayload {
			passkey_public_key: [0u8; 33],
			verifiable_passkey_signature: VerifiablePasskeySignature {
				signature: PasskeySignature::default(),
				client_data_json: PasskeyClientDataJson::default(),
				authenticator_data: PasskeyAuthenticatorData::default(),
			},
			passkey_call: call,
		};

		// assert
		assert_ok!(Passkey::proxy(RuntimeOrigin::none(), payload));
	});
}

#[test]
fn validate_unsigned_with_unsupported_call_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let (test_account_1_key_pair, _) = sr25519::Pair::generate();
		let signature: MultiSignature = test_account_1_key_pair.sign(b"sdsds").into();
		let call: PasskeyCall<Test> = PasskeyCall {
			account_id: test_account_1_key_pair.public().into(),
			account_nonce: 3,
			account_ownership_proof: signature,
			// remark is an unsupported call
			call: Box::new(RuntimeCall::System(SystemCall::remark { remark: vec![1, 2, 3u8] })),
		};
		let payload = PasskeyPayload {
			passkey_public_key: [0u8; 33],
			verifiable_passkey_signature: VerifiablePasskeySignature {
				signature: PasskeySignature::default(),
				client_data_json: PasskeyClientDataJson::default(),
				authenticator_data: PasskeyAuthenticatorData::default(),
			},
			passkey_call: call,
		};

		// act
		let v = Passkey::validate_unsigned(TransactionSource::InBlock, &Call::proxy { payload });

		// assert
		let err: TransactionValidityError = InvalidTransaction::Call.into();
		assert_err!(v, err);
	});
}

#[test]
fn validate_unsigned_with_used_nonce_should_fail_with_stale() {
	new_test_ext().execute_with(|| {
		// arrange
		let (test_account_1_key_pair, _) = sr25519::Pair::generate();
		let (test_account_2_key_pair, _) = sr25519::Pair::generate();
		let who: <Test as frame_system::Config>::AccountId =
			test_account_1_key_pair.public().into();
		let mut account = frame_system::Account::<Test>::get(&who);
		account.nonce += 1;
		frame_system::Account::<Test>::insert(who, account);

		let signature: MultiSignature = test_account_1_key_pair.sign(b"sdsds").into();
		let call: PasskeyCall<Test> = PasskeyCall {
			account_id: test_account_1_key_pair.public().into(),
			account_nonce: 0,
			account_ownership_proof: signature,
			call: Box::new(RuntimeCall::Balances(BalancesCall::transfer_allow_death {
				dest: test_account_2_key_pair.public().into(),
				value: 10000,
			})),
		};
		let payload = PasskeyPayload {
			passkey_public_key: [0u8; 33],
			verifiable_passkey_signature: VerifiablePasskeySignature {
				signature: PasskeySignature::default(),
				client_data_json: PasskeyClientDataJson::default(),
				authenticator_data: PasskeyAuthenticatorData::default(),
			},
			passkey_call: call,
		};

		// act
		let v = Passkey::validate_unsigned(TransactionSource::InBlock, &Call::proxy { payload });

		// assert
		let err: TransactionValidityError = InvalidTransaction::Stale.into();
		assert_err!(v, err);
	});
}

#[test]
fn validate_unsigned_with_correct_nonce_should_work() {
	new_test_ext().execute_with(|| {
		// arrange
		let (test_account_1_key_pair, _) = sr25519::Pair::generate();
		let (test_account_2_key_pair, _) = sr25519::Pair::generate();
		let who: <Test as frame_system::Config>::AccountId =
			test_account_1_key_pair.public().into();
		let mut account = frame_system::Account::<Test>::get(&who);
		account.nonce += 1;
		frame_system::Account::<Test>::insert(who.clone(), account);

		let signature: MultiSignature = test_account_1_key_pair.sign(b"sdsds").into();
		let call: PasskeyCall<Test> = PasskeyCall {
			account_id: test_account_1_key_pair.public().into(),
			account_nonce: 2,
			account_ownership_proof: signature,
			call: Box::new(RuntimeCall::Balances(BalancesCall::transfer_allow_death {
				dest: test_account_2_key_pair.public().into(),
				value: 10000,
			})),
		};
		let payload = PasskeyPayload {
			passkey_public_key: [0u8; 33],
			verifiable_passkey_signature: VerifiablePasskeySignature {
				signature: PasskeySignature::default(),
				client_data_json: PasskeyClientDataJson::default(),
				authenticator_data: PasskeyAuthenticatorData::default(),
			},
			passkey_call: call,
		};

		// act
		let v = Passkey::validate_unsigned(TransactionSource::InBlock, &Call::proxy { payload });

		// assert
		assert!(v.is_ok());

		assert_eq!(
			v,
			Ok(ValidTransaction {
				priority: 0,
				requires: vec![Encode::encode(&(who.clone(), 1u64))],
				provides: vec![Encode::encode(&(who, 2u64))],
				longevity: TransactionLongevity::max_value(),
				propagate: true,
			})
		);
	});
}

#[test]
fn pre_dispatch_unsigned_with_used_nonce_should_fail_with_stale() {
	new_test_ext().execute_with(|| {
		// arrange
		let (test_account_1_key_pair, _) = sr25519::Pair::generate();
		let (test_account_2_key_pair, _) = sr25519::Pair::generate();
		let who: <Test as frame_system::Config>::AccountId =
			test_account_1_key_pair.public().into();
		let mut account = frame_system::Account::<Test>::get(&who);
		account.nonce += 1;
		frame_system::Account::<Test>::insert(who, account);

		let signature: MultiSignature = test_account_1_key_pair.sign(b"sdsds").into();
		let call: PasskeyCall<Test> = PasskeyCall {
			account_id: test_account_1_key_pair.public().into(),
			account_nonce: 0,
			account_ownership_proof: signature,
			call: Box::new(RuntimeCall::Balances(BalancesCall::transfer_allow_death {
				dest: test_account_2_key_pair.public().into(),
				value: 10000,
			})),
		};
		let payload = PasskeyPayload {
			passkey_public_key: [0u8; 33],
			verifiable_passkey_signature: VerifiablePasskeySignature {
				signature: PasskeySignature::default(),
				client_data_json: PasskeyClientDataJson::default(),
				authenticator_data: PasskeyAuthenticatorData::default(),
			},
			passkey_call: call,
		};

		// act
		let v = Passkey::pre_dispatch(&Call::proxy { payload });

		// assert
		let err: TransactionValidityError = InvalidTransaction::Stale.into();
		assert_err!(v, err);
	});
}

#[test]
fn pre_dispatch_unsigned_with_future_nonce_should_fail_with_future() {
	new_test_ext().execute_with(|| {
		// arrange
		let (test_account_1_key_pair, _) = sr25519::Pair::generate();
		let (test_account_2_key_pair, _) = sr25519::Pair::generate();
		let signature: MultiSignature = test_account_1_key_pair.sign(b"sdsds").into();
		let call: PasskeyCall<Test> = PasskeyCall {
			account_id: test_account_1_key_pair.public().into(),
			account_nonce: 2,
			account_ownership_proof: signature,
			call: Box::new(RuntimeCall::Balances(BalancesCall::transfer_allow_death {
				dest: test_account_2_key_pair.public().into(),
				value: 10000,
			})),
		};
		let payload = PasskeyPayload {
			passkey_public_key: [0u8; 33],
			verifiable_passkey_signature: VerifiablePasskeySignature {
				signature: PasskeySignature::default(),
				client_data_json: PasskeyClientDataJson::default(),
				authenticator_data: PasskeyAuthenticatorData::default(),
			},
			passkey_call: call,
		};

		// act
		let v = Passkey::pre_dispatch(&Call::proxy { payload });

		// assert
		let err: TransactionValidityError = InvalidTransaction::Future.into();
		assert_err!(v, err);
	});
}

#[test]
fn pre_dispatch_unsigned_should_increment_nonce_on_success() {
	new_test_ext().execute_with(|| {
		// arrange
		let (test_account_1_key_pair, _) = sr25519::Pair::generate();
		let account_1_pk: <Test as frame_system::Config>::AccountId =
			test_account_1_key_pair.public().into();
		let (test_account_2_key_pair, _) = sr25519::Pair::generate();
		let signature: MultiSignature = test_account_1_key_pair.sign(b"sdsds").into();
		let call: PasskeyCall<Test> = PasskeyCall {
			account_id: account_1_pk.clone(),
			account_nonce: 0,
			account_ownership_proof: signature,
			call: Box::new(RuntimeCall::Balances(BalancesCall::transfer_allow_death {
				dest: test_account_2_key_pair.public().into(),
				value: 10000,
			})),
		};
		let payload = PasskeyPayload {
			passkey_public_key: [0u8; 33],
			verifiable_passkey_signature: VerifiablePasskeySignature {
				signature: PasskeySignature::default(),
				client_data_json: PasskeyClientDataJson::default(),
				authenticator_data: PasskeyAuthenticatorData::default(),
			},
			passkey_call: call,
		};

		// act assert
		assert_ok!(Passkey::pre_dispatch(&Call::proxy { payload }));

		let account = frame_system::Account::<Test>::get(&account_1_pk);
		assert_eq!(account.nonce, <Test as frame_system::Config>::Nonce::one());
	});
}
