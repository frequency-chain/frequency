//! Unit tests for the passkey module.
use super::*;
use frame_support::{assert_noop, assert_ok};
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
