//! Unit tests for the passkey module.
use super::*;
use common_primitives::utils::wrap_binary_data;
use frame_support::{assert_noop, assert_ok};
use frame_system::{Call as SystemCall, RawOrigin};
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
			1305343182u32.into()
		));
		let balance_after = Balances::free_balance(&test_account_1_key_pair.public().into());
		assert_eq!(balance_after, 1305343182);
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
	});
}

#[test]
fn test_proxy_call_with_low_funds_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let (test_account_1_key_pair, _) = sr25519::Pair::generate();
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
		assert_eq!(res, InvalidTransaction::Payment.into());
	});
}
