//! Unit tests for the passkey module.
use super::*;
use crate::mock::Passkey;
use common_primitives::utils::wrap_binary_data;
use frame_support::{assert_err, assert_noop, assert_ok};
use frame_system::{Call as SystemCall, RawOrigin};
use mock::*;
use pallet_balances::Call as BalancesCall;
use sp_core::{sr25519, Pair};
use sp_runtime::{traits::One, DispatchError::BadOrigin, MultiSignature};

#[test]
fn proxy_call_with_signed_origin_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let (test_account_1_key_pair, _) = sr25519::Pair::generate();
		let (test_account_2_key_pair, _) = sr25519::Pair::generate();
		let passkey_public_key = [0u8; 33];
		let wrapped_binary = wrap_binary_data(passkey_public_key.to_vec());
		let signature: MultiSignature =
			test_account_1_key_pair.sign(wrapped_binary.as_slice()).into();
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
			passkey_public_key,
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
		let passkey_public_key = [0u8; 33];
		let wrapped_binary = wrap_binary_data(passkey_public_key.to_vec());
		let signature: MultiSignature =
			test_account_1_key_pair.sign(wrapped_binary.as_slice()).into();
		let call: PasskeyCall<Test> = PasskeyCall {
			account_id: test_account_1_key_pair.public().into(),
			account_nonce: 3,
			account_ownership_proof: signature,
			call: Box::new(RuntimeCall::System(SystemCall::remark { remark: vec![1, 2, 3u8] })),
		};
		let payload = PasskeyPayload {
			passkey_public_key,
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
fn test_proxy_call_with_bad_signature_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let (test_account_1_key_pair, _) = sr25519::Pair::generate();
		// fund the account with 10 units
		assert_ok!(Balances::force_set_balance(
			RawOrigin::Root.into(),
			test_account_1_key_pair.public().into(),
			1000000000u32.into()
		));
		let balance_after = Balances::free_balance(&test_account_1_key_pair.public().into());
		assert_eq!(balance_after, 1000000000);
		let passkey_public_key = [0u8; 33];
		let wrapped_binary = wrap_binary_data("bad data".as_bytes().to_vec());
		let signature: MultiSignature =
			test_account_1_key_pair.sign(wrapped_binary.as_slice()).into();
		let call: PasskeyCall<Test> = PasskeyCall {
			account_id: test_account_1_key_pair.public().into(),
			account_nonce: 3,
			account_ownership_proof: signature,
			call: Box::new(RuntimeCall::System(SystemCall::remark { remark: vec![1, 2, 3u8] })),
		};
		let payload = PasskeyPayload {
			passkey_public_key,
			verifiable_passkey_signature: VerifiablePasskeySignature {
				signature: PasskeySignature::default(),
				client_data_json: PasskeyClientDataJson::default(),
				authenticator_data: PasskeyAuthenticatorData::default(),
			},
			passkey_call: call,
		};
		let res = Passkey::validate_unsigned(TransactionSource::InBlock, &Call::proxy { payload });
		// assert
		assert_eq!(res, InvalidTransaction::BadSigner.into());
		let balance_after = Balances::free_balance(&test_account_1_key_pair.public().into());
		assert_eq!(balance_after, 1000000000);
	});
}

#[test]
fn test_proxy_call_with_low_funds_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let (test_account_1_key_pair, _) = sr25519::Pair::generate();
		let passkey_public_key = [0u8; 33];
		let wrapped_binary = wrap_binary_data(passkey_public_key.to_vec());
		let signature: MultiSignature =
			test_account_1_key_pair.sign(wrapped_binary.as_slice()).into();
		let call: PasskeyCall<Test> = PasskeyCall {
			account_id: test_account_1_key_pair.public().into(),
			account_nonce: 3,
			account_ownership_proof: signature,
			call: Box::new(RuntimeCall::System(SystemCall::remark { remark: vec![1, 2, 3u8] })),
		};
		let payload = PasskeyPayload {
			passkey_public_key,
			verifiable_passkey_signature: VerifiablePasskeySignature {
				signature: PasskeySignature::default(),
				client_data_json: PasskeyClientDataJson::default(),
				authenticator_data: PasskeyAuthenticatorData::default(),
			},
			passkey_call: call,
		};
		let res = Passkey::validate_unsigned(TransactionSource::InBlock, &Call::proxy { payload });
		// assert
		assert_eq!(res, InvalidTransaction::Payment.into());
	});
}

#[test]
fn test_proxy_call_with_funds_should_pass() {
	new_test_ext().execute_with(|| {
		// arrange
		let (test_account_1_key_pair, _) = sr25519::Pair::generate();
		// fund the account with 10 units
		assert_ok!(Balances::force_set_balance(
			RawOrigin::Root.into(),
			test_account_1_key_pair.public().into(),
			1000000000u32.into()
		));
		let balance_after = Balances::free_balance(&test_account_1_key_pair.public().into());
		assert_eq!(balance_after, 1000000000);
		let passkey_public_key = [0u8; 33];
		let wrapped_binary = wrap_binary_data(passkey_public_key.to_vec());
		let signature: MultiSignature =
			test_account_1_key_pair.sign(wrapped_binary.as_slice()).into();
		let call: PasskeyCall<Test> = PasskeyCall {
			account_id: test_account_1_key_pair.public().into(),
			account_nonce: 3,
			account_ownership_proof: signature,
			call: Box::new(RuntimeCall::System(SystemCall::remark { remark: vec![1, 2, 3u8] })),
		};
		let payload = PasskeyPayload {
			passkey_public_key,
			verifiable_passkey_signature: VerifiablePasskeySignature {
				signature: PasskeySignature::default(),
				client_data_json: PasskeyClientDataJson::default(),
				authenticator_data: PasskeyAuthenticatorData::default(),
			},
			passkey_call: call,
		};
		let res = Passkey::validate_unsigned(TransactionSource::InBlock, &Call::proxy { payload });
		// assert
		assert!(res.is_ok());
	});
}

#[test]
fn test_pre_dispatch_with_funds_should_pass() {
	new_test_ext().execute_with(|| {
		// arrange
		let (test_account_1_key_pair, _) = sr25519::Pair::generate();
		// fund the account with 10 units
		assert_ok!(Balances::force_set_balance(
			RawOrigin::Root.into(),
			test_account_1_key_pair.public().into(),
			1000000000u32.into()
		));
		let balance_after = Balances::free_balance(&test_account_1_key_pair.public().into());
		assert_eq!(balance_after, 1000000000);
		let passkey_public_key = [0u8; 33];
		let wrapped_binary = wrap_binary_data(passkey_public_key.to_vec());
		let signature: MultiSignature =
			test_account_1_key_pair.sign(wrapped_binary.as_slice()).into();
		let call: PasskeyCall<Test> = PasskeyCall {
			account_id: test_account_1_key_pair.public().into(),
			account_nonce: 0,
			account_ownership_proof: signature,
			call: Box::new(RuntimeCall::System(SystemCall::remark { remark: vec![1, 2, 3u8] })),
		};
		let payload = PasskeyPayload {
			passkey_public_key,
			verifiable_passkey_signature: VerifiablePasskeySignature {
				signature: PasskeySignature::default(),
				client_data_json: PasskeyClientDataJson::default(),
				authenticator_data: PasskeyAuthenticatorData::default(),
			},
			passkey_call: call,
		};
		let res = Passkey::pre_dispatch(&Call::proxy { payload });
		log::error!("res: {:?}", res);
		// assert
		assert!(res.is_ok());
	});
}

#[test]
fn test_pre_dispatch_with_low_funds_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let (test_account_1_key_pair, _) = sr25519::Pair::generate();
		let passkey_public_key = [0u8; 33];
		let wrapped_binary = wrap_binary_data(passkey_public_key.to_vec());
		let signature: MultiSignature =
			test_account_1_key_pair.sign(wrapped_binary.as_slice()).into();
		let call: PasskeyCall<Test> = PasskeyCall {
			account_id: test_account_1_key_pair.public().into(),
			account_nonce: 3,
			account_ownership_proof: signature,
			call: Box::new(RuntimeCall::System(SystemCall::remark { remark: vec![1, 2, 3u8] })),
		};
		let payload = PasskeyPayload {
			passkey_public_key,
			verifiable_passkey_signature: VerifiablePasskeySignature {
				signature: PasskeySignature::default(),
				client_data_json: PasskeyClientDataJson::default(),
				authenticator_data: PasskeyAuthenticatorData::default(),
			},
			passkey_call: call,
		};
		let res = Passkey::pre_dispatch(&Call::proxy { payload });

		// assert
		assert_eq!(res, Err(InvalidTransaction::Payment.into()));
	});
}

#[test]
fn fee_removed_on_successful_validation() {
	new_test_ext().execute_with(|| {
		// arrange
		let (test_account_1_key_pair, _) = sr25519::Pair::generate();
		let (test_account_2_key_pair, _) = sr25519::Pair::generate();
		let account_id: <Test as frame_system::Config>::AccountId =
			test_account_1_key_pair.public().into();
		let destination_id = test_account_2_key_pair.public().into();
		// Fund the account
		assert_ok!(Balances::force_set_balance(
			RawOrigin::Root.into(),
			account_id.clone(),
			1000000000u32.into()
		));
		let initial_balance = Balances::free_balance(&account_id);

		let passkey_public_key = [0u8; 33];
		let wrapped_binary = wrap_binary_data(passkey_public_key.to_vec());
		let signature: MultiSignature = test_account_1_key_pair.sign(&wrapped_binary).into();

		let call: PasskeyCall<Test> = PasskeyCall {
			account_id: account_id.clone(),
			account_nonce: 0,
			account_ownership_proof: signature,
			call: Box::new(RuntimeCall::Balances(BalancesCall::transfer_allow_death {
				dest: destination_id,
				value: 10000,
			})),
		};
		let payload = PasskeyPayload {
			passkey_public_key,
			verifiable_passkey_signature: VerifiablePasskeySignature {
				signature: PasskeySignature::default(),
				client_data_json: PasskeyClientDataJson::default(),
				authenticator_data: PasskeyAuthenticatorData::default(),
			},
			passkey_call: call,
		};

		// act
		let res = Passkey::validate_unsigned(TransactionSource::InBlock, &Call::proxy { payload });

		// assert
		assert!(res.is_ok());
		let final_balance = Balances::free_balance(&account_id);
		assert!(final_balance < initial_balance);
	});
}

#[test]
fn fee_withdrawn_for_failed_call() {
	new_test_ext().execute_with(|| {
		// arrange
		let (test_account_1_key_pair, _) = sr25519::Pair::generate();
		let account_id: <Test as frame_system::Config>::AccountId =
			test_account_1_key_pair.public().into();
		// Fund the account
		assert_ok!(Balances::force_set_balance(
			RawOrigin::Root.into(),
			account_id.clone(),
			1000000000u32.into()
		));
		let initial_balance = Balances::free_balance(&account_id);

		let passkey_public_key = [0u8; 33];
		let wrapped_binary = wrap_binary_data(passkey_public_key.to_vec());
		let signature: MultiSignature = test_account_1_key_pair.sign(&wrapped_binary).into();

		let call: PasskeyCall<Test> = PasskeyCall {
			account_id: account_id.clone(),
			account_nonce: 0,
			account_ownership_proof: signature,
			call: Box::new(RuntimeCall::Balances(BalancesCall::transfer_allow_death {
				dest: account_id.clone(),
				value: 1000000000,
			})),
		};
		let payload = PasskeyPayload {
			passkey_public_key,
			verifiable_passkey_signature: VerifiablePasskeySignature {
				signature: PasskeySignature::default(),
				client_data_json: PasskeyClientDataJson::default(),
				authenticator_data: PasskeyAuthenticatorData::default(),
			},
			passkey_call: call,
		};

		// act
		let res = Passkey::validate_unsigned(
			TransactionSource::InBlock,
			&Call::proxy { payload: payload.clone() },
		);

		// assert
		assert!(res.is_ok());

		// Call the proxy
		let res = Passkey::proxy(RuntimeOrigin::none(), payload);
		assert!(res.is_err());
		let final_balance = Balances::free_balance(&account_id);
		assert!(final_balance < initial_balance);
	});
}

#[test]
fn fee_not_removed_on_failed_validation() {
	new_test_ext().execute_with(|| {
		// arrange
		let (test_account_1_key_pair, _) = sr25519::Pair::generate();
		let account_id: <Test as frame_system::Config>::AccountId =
			test_account_1_key_pair.public().into();
		// Fund the account
		assert_ok!(Balances::force_set_balance(
			RawOrigin::Root.into(),
			account_id.clone(),
			1000000000u32.into()
		));
		let initial_balance = Balances::free_balance(&account_id);

		let passkey_public_key = [0u8; 33];
		let wrapped_binary = wrap_binary_data("invalid data".as_bytes().to_vec());
		let signature: MultiSignature = test_account_1_key_pair.sign(&wrapped_binary).into();

		let call: PasskeyCall<Test> = PasskeyCall {
			account_id: account_id.clone(),
			account_nonce: 0,
			account_ownership_proof: signature,
			call: Box::new(RuntimeCall::System(SystemCall::remark { remark: vec![1, 2, 3u8] })),
		};
		let payload = PasskeyPayload {
			passkey_public_key,
			verifiable_passkey_signature: VerifiablePasskeySignature {
				signature: PasskeySignature::default(),
				client_data_json: PasskeyClientDataJson::default(),
				authenticator_data: PasskeyAuthenticatorData::default(),
			},
			passkey_call: call,
		};

		// act
		let res = Passkey::validate_unsigned(TransactionSource::InBlock, &Call::proxy { payload });

		// assert
		assert_eq!(res, InvalidTransaction::BadSigner.into());
		let final_balance = Balances::free_balance(&account_id);
		assert_eq!(initial_balance, final_balance);
	});
}

#[test]
fn validate_unsigned_with_unsupported_call_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let (test_account_1_key_pair, _) = sr25519::Pair::generate();
		let wrapped_binary = wrap_binary_data("data".as_bytes().to_vec());
		let signature: MultiSignature = test_account_1_key_pair.sign(&wrapped_binary).into();
		let call: PasskeyCall<Test> = PasskeyCall {
			account_id: test_account_1_key_pair.public().into(),
			account_nonce: 3,
			account_ownership_proof: signature,
			// remark is an unsupported call
			call: Box::new(RuntimeCall::System(SystemCall::remark_with_event {
				remark: vec![1, 2, 3u8],
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
		// Fund
		assert_ok!(Balances::force_set_balance(
			RawOrigin::Root.into(),
			test_account_1_key_pair.public().into(),
			1000000000u32.into()
		));
		let who: <Test as frame_system::Config>::AccountId =
			test_account_1_key_pair.public().into();
		let mut account = frame_system::Account::<Test>::get(&who);
		account.nonce += 1;
		frame_system::Account::<Test>::insert(who, account);

		let passkey_public_key = [0u8; 33];
		let wrapped_binary = wrap_binary_data(passkey_public_key.to_vec());
		let signature: MultiSignature = test_account_1_key_pair.sign(&wrapped_binary).into();

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
			passkey_public_key,
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
		// Fund
		assert_ok!(Balances::force_set_balance(
			RawOrigin::Root.into(),
			test_account_1_key_pair.public().into(),
			1000000000u32.into()
		));
		let who: <Test as frame_system::Config>::AccountId =
			test_account_1_key_pair.public().into();
		let mut account = frame_system::Account::<Test>::get(&who);
		account.nonce += 1;
		frame_system::Account::<Test>::insert(who.clone(), account);

		let passkey_public_key = [0u8; 33];
		let wrapped_binary = wrap_binary_data(passkey_public_key.to_vec());
		let signature: MultiSignature = test_account_1_key_pair.sign(&wrapped_binary).into();

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
			passkey_public_key,
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
		// Fund
		assert_ok!(Balances::force_set_balance(
			RawOrigin::Root.into(),
			test_account_1_key_pair.public().into(),
			1000000000u32.into()
		));
		let who: <Test as frame_system::Config>::AccountId =
			test_account_1_key_pair.public().into();
		let mut account = frame_system::Account::<Test>::get(&who);
		account.nonce += 1;
		frame_system::Account::<Test>::insert(who, account);

		let passkey_public_key = [0u8; 33];
		let wrapped_binary = wrap_binary_data(passkey_public_key.to_vec());
		let signature: MultiSignature = test_account_1_key_pair.sign(&wrapped_binary).into();

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
			passkey_public_key,
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
		// Fund
		assert_ok!(Balances::force_set_balance(
			RawOrigin::Root.into(),
			test_account_1_key_pair.public().into(),
			1000000000u32.into()
		));
		let passkey_public_key = [0u8; 33];
		let wrapped_binary = wrap_binary_data(passkey_public_key.to_vec());
		let signature: MultiSignature = test_account_1_key_pair.sign(&wrapped_binary).into();
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
			passkey_public_key,
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
		// Fund
		assert_ok!(Balances::force_set_balance(
			RawOrigin::Root.into(),
			account_1_pk.clone(),
			1000000000u32.into()
		));
		let (test_account_2_key_pair, _) = sr25519::Pair::generate();
		let passkey_public_key = [0u8; 33];
		let wrapped_binary = wrap_binary_data(passkey_public_key.to_vec());
		let signature: MultiSignature = test_account_1_key_pair.sign(&wrapped_binary).into();
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
			passkey_public_key,
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
