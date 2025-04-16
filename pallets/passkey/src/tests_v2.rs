//! Unit tests for the passkey module.
use super::*;
use crate::mock::Passkey;
use common_primitives::utils::wrap_binary_data;
use frame_support::{assert_err, assert_noop, assert_ok, dispatch::RawOrigin};
use frame_system::{limits::BlockLength, Call as SystemCall};
use mock::*;

use crate::test_common::{
	constants::{AUTHENTICATOR_DATA, REPLACED_CLIENT_DATA_JSON},
	utilities::*,
};
use pallet_balances::Call as BalancesCall;
use sp_core::{bytes::from_hex, ecdsa, sr25519, sr25519::Public, Pair};
use sp_runtime::{traits::One, DispatchError::BadOrigin};

struct TestPasskeyPayloadBuilder {
	secret: p256::SecretKey,
	key_pair: sr25519::Pair,
	passkey_public_key: PasskeyPublicKey,
	payload_to_sign: Vec<u8>,
	nonce: u32,
	call: <Test as Config>::RuntimeCall,
	invalid_passkey_signature: bool,
}

impl TestPasskeyPayloadBuilder {
	pub fn new() -> Self {
		let (key_pair, _) = sr25519::Pair::generate();
		Self {
			secret: p256::SecretKey::from_slice(&[
				1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8,
			])
			.unwrap(),
			key_pair,
			passkey_public_key: PasskeyPublicKey([0u8; 33]),
			payload_to_sign: vec![],
			nonce: 0u32,
			call: RuntimeCall::System(SystemCall::remark { remark: vec![1, 2, 3u8] }),
			invalid_passkey_signature: false,
		}
	}

	pub fn with_a_valid_passkey(mut self) -> Self {
		self.passkey_public_key = get_p256_public_key(&self.secret).unwrap();
		self
	}

	pub fn with_custom_payload(mut self, payload: Vec<u8>) -> Self {
		self.payload_to_sign = payload;
		self
	}

	pub fn with_passkey_as_payload(mut self) -> Self {
		self.payload_to_sign = self.passkey_public_key.inner().to_vec();
		self
	}

	pub fn with_account_nonce(mut self, nonce: u32) -> Self {
		self.nonce = nonce;
		self
	}

	pub fn with_call(mut self, call: <Test as crate::Config>::RuntimeCall) -> Self {
		self.call = call;
		self
	}

	pub fn with_invalid_passkey_signature(mut self) -> Self {
		self.invalid_passkey_signature = true;
		self
	}

	pub fn with_funded_account(self, amount: u64) -> Self {
		assert_ok!(Balances::force_set_balance(
			RawOrigin::Root.into(),
			self.key_pair.public().into(),
			amount
		));
		self
	}

	pub fn build(&self) -> (PasskeyPayloadV2<Test>, Public) {
		let wrapped_binary = wrap_binary_data(self.payload_to_sign.clone());
		let signature: MultiSignature = self.key_pair.sign(wrapped_binary.as_slice()).into();
		let client_data = base64_url::decode(REPLACED_CLIENT_DATA_JSON).unwrap();
		let authenticator = base64_url::decode(AUTHENTICATOR_DATA).unwrap();
		let bad_authenticator = b"bad_auth".to_vec();
		let call: PasskeyCallV2<Test> = PasskeyCallV2 {
			account_id: self.key_pair.public().into(),
			account_nonce: self.nonce.into(),
			call: Box::new(self.call.clone()),
		};
		let passkey_signature = passkey_sign(
			&self.secret,
			&call.encode(),
			&client_data,
			match self.invalid_passkey_signature {
				true => &bad_authenticator,
				false => &authenticator,
			},
		)
		.unwrap();
		let payload = PasskeyPayloadV2 {
			passkey_public_key: self.passkey_public_key.clone(),
			verifiable_passkey_signature: VerifiablePasskeySignature {
				signature: passkey_signature,
				client_data_json: client_data.try_into().unwrap(),
				authenticator_data: authenticator.try_into().unwrap(),
			},
			account_ownership_proof: signature,
			passkey_call: call,
		};
		(payload, self.key_pair.public())
	}
}

#[test]
fn proxy_call_with_signed_origin_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let (test_account_2_key_pair, _) = sr25519::Pair::generate();
		let (payload, account_pk) = TestPasskeyPayloadBuilder::new()
			.with_passkey_as_payload()
			.with_call(RuntimeCall::Balances(BalancesCall::transfer_allow_death {
				dest: test_account_2_key_pair.public().into(),
				value: 100,
			}))
			.build();

		// assert
		assert_noop!(
			Passkey::proxy_v2(RuntimeOrigin::signed(account_pk.into()), payload),
			BadOrigin
		);
	});
}

#[test]
fn proxy_call_with_unsigned_origin_should_work() {
	new_test_ext().execute_with(|| {
		// arrange
		let (payload, _) = TestPasskeyPayloadBuilder::new()
			.with_passkey_as_payload()
			.with_call(RuntimeCall::System(SystemCall::remark { remark: vec![1, 2, 3u8] }))
			.build();

		// assert
		assert_ok!(Passkey::proxy_v2(RuntimeOrigin::none(), payload));
	});
}

#[test]
fn validate_unsigned_with_bad_account_signature_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let (payload, _) = TestPasskeyPayloadBuilder::new()
			.with_a_valid_passkey()
			.with_custom_payload("bad data".as_bytes().to_vec())
			.with_call(RuntimeCall::System(SystemCall::remark { remark: vec![1, 2, 3u8] }))
			.with_funded_account(10000000000)
			.build();

		let res =
			Passkey::validate_unsigned(TransactionSource::InBlock, &Call::proxy_v2 { payload });
		// assert
		assert_eq!(res, InvalidTransaction::BadSigner.into());
	});
}

#[test]
fn validate_unsigned_with_bad_passkey_signature_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let amount = 10000000000;
		let (payload, _) = TestPasskeyPayloadBuilder::new()
			.with_a_valid_passkey()
			.with_passkey_as_payload()
			.with_call(RuntimeCall::System(SystemCall::remark { remark: vec![1, 2, 3u8] }))
			.with_funded_account(amount)
			.with_invalid_passkey_signature()
			.build();

		let res =
			Passkey::validate_unsigned(TransactionSource::InBlock, &Call::proxy_v2 { payload });
		// assert
		assert_eq!(res, InvalidTransaction::BadSigner.into());
	});
}

#[test]
fn validate_unsigned_with_low_funds_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let (payload, _) = TestPasskeyPayloadBuilder::new()
			.with_a_valid_passkey()
			.with_passkey_as_payload()
			.with_call(RuntimeCall::System(SystemCall::remark { remark: vec![1, 2, 3u8] }))
			.build();

		// act
		let res =
			Passkey::validate_unsigned(TransactionSource::InBlock, &Call::proxy_v2 { payload });
		// assert
		assert_eq!(res, InvalidTransaction::Payment.into());
	});
}

#[test]
fn validate_unsigned_with_funds_should_pass() {
	new_test_ext().execute_with(|| {
		// arrange
		let (payload, _) = TestPasskeyPayloadBuilder::new()
			.with_a_valid_passkey()
			.with_passkey_as_payload()
			.with_call(RuntimeCall::System(SystemCall::remark { remark: vec![1, 2, 3u8] }))
			.with_funded_account(10000000000)
			.build();

		// act
		let res =
			Passkey::validate_unsigned(TransactionSource::InBlock, &Call::proxy_v2 { payload });

		// assert
		assert!(res.is_ok());
	});
}

#[test]
fn pre_dispatch_with_funds_should_pass() {
	new_test_ext().execute_with(|| {
		// arrange
		let (payload, _) = TestPasskeyPayloadBuilder::new()
			.with_a_valid_passkey()
			.with_passkey_as_payload()
			.with_call(RuntimeCall::System(SystemCall::remark { remark: vec![1, 2, 3u8] }))
			.with_funded_account(10000000000)
			.build();

		// act
		let res = Passkey::pre_dispatch(&Call::proxy_v2 { payload });

		// assert
		assert!(res.is_ok());
	});
}

#[test]
fn pre_dispatch_with_low_funds_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let (payload, _) = TestPasskeyPayloadBuilder::new()
			.with_a_valid_passkey()
			.with_passkey_as_payload()
			.with_call(RuntimeCall::System(SystemCall::remark { remark: vec![1, 2, 3u8] }))
			.build();

		// act
		let res = Passkey::pre_dispatch(&Call::proxy_v2 { payload });

		// assert
		assert_err!(res, InvalidTransaction::Payment);
	});
}

#[test]
fn validate_unsigned_fee_should_not_get_removed_on_successful_validation() {
	new_test_ext().execute_with(|| {
		// arrange
		let (test_account_2_key_pair, _) = sr25519::Pair::generate();
		let (payload, account_pk) = TestPasskeyPayloadBuilder::new()
			.with_a_valid_passkey()
			.with_passkey_as_payload()
			.with_call(RuntimeCall::Balances(BalancesCall::transfer_allow_death {
				dest: test_account_2_key_pair.public().into(),
				value: 100,
			}))
			.with_funded_account(10000000000)
			.build();

		let account_id: <Test as frame_system::Config>::AccountId = account_pk.into();
		let initial_balance = Balances::free_balance(&account_id);

		// act
		let res =
			Passkey::validate_unsigned(TransactionSource::InBlock, &Call::proxy_v2 { payload });

		// assert
		assert!(res.is_ok());
		let final_balance = Balances::free_balance(&account_id);
		assert_eq!(final_balance, initial_balance);
	});
}

#[test]
fn pre_dispatch_fee_should_get_removed_on_successful_validation() {
	new_test_ext().execute_with(|| {
		// arrange
		let (test_account_2_key_pair, _) = sr25519::Pair::generate();
		let (payload, account_pk) = TestPasskeyPayloadBuilder::new()
			.with_a_valid_passkey()
			.with_passkey_as_payload()
			.with_call(RuntimeCall::Balances(BalancesCall::transfer_allow_death {
				dest: test_account_2_key_pair.public().into(),
				value: 100,
			}))
			.with_funded_account(10000000000)
			.build();

		let account_id: <Test as frame_system::Config>::AccountId = account_pk.into();
		let initial_balance = Balances::free_balance(&account_id);

		// act
		let res = Passkey::pre_dispatch(&Call::proxy_v2 { payload });

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
		let amount = 10000000000;
		let (test_account_2_key_pair, _) = sr25519::Pair::generate();
		let (payload, account_pk) = TestPasskeyPayloadBuilder::new()
			.with_a_valid_passkey()
			.with_passkey_as_payload()
			.with_call(RuntimeCall::Balances(BalancesCall::transfer_allow_death {
				dest: test_account_2_key_pair.public().into(),
				value: amount,
			}))
			.with_funded_account(amount)
			.build();

		let account_id: <Test as frame_system::Config>::AccountId = account_pk.into();
		let initial_balance = Balances::free_balance(&account_id);

		// act
		let validate_result = Passkey::pre_dispatch(&Call::proxy_v2 { payload: payload.clone() });
		let extrinsic_result = Passkey::proxy_v2(RuntimeOrigin::none(), payload);

		// assert
		assert!(validate_result.is_ok());
		assert!(extrinsic_result.is_err());
		let final_balance = Balances::free_balance(&account_id);
		assert!(final_balance < initial_balance);
	});
}

#[test]
fn validate_unsigned_with_unsupported_call_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let (payload, _) = TestPasskeyPayloadBuilder::new()
			.with_a_valid_passkey()
			.with_passkey_as_payload()
			// remark_with_event is an unsupported call
			.with_call(RuntimeCall::System(SystemCall::remark_with_event {
				remark: vec![1, 2, 3u8],
			}))
			.build();

		// act
		let v = Passkey::validate_unsigned(TransactionSource::InBlock, &Call::proxy_v2 { payload });

		// assert
		assert_err!(v, InvalidTransaction::Call);
	});
}

#[test]
fn validate_unsigned_with_used_nonce_should_fail_with_stale() {
	new_test_ext().execute_with(|| {
		// arrange
		let (test_account_2_key_pair, _) = sr25519::Pair::generate();
		let (payload, account_pk) = TestPasskeyPayloadBuilder::new()
			.with_a_valid_passkey()
			.with_passkey_as_payload()
			.with_call(RuntimeCall::Balances(BalancesCall::transfer_allow_death {
				dest: test_account_2_key_pair.public().into(),
				value: 10000,
			}))
			.with_funded_account(10000000000)
			.with_account_nonce(0)
			.build();

		let who: <Test as frame_system::Config>::AccountId = account_pk.into();
		let mut account = frame_system::Account::<Test>::get(&who);
		account.nonce += 1;
		frame_system::Account::<Test>::insert(who, account);

		// act
		let v = Passkey::validate_unsigned(TransactionSource::InBlock, &Call::proxy_v2 { payload });

		// assert
		assert_err!(v, InvalidTransaction::Stale);
	});
}

#[test]
fn validate_unsigned_with_correct_nonce_should_work() {
	new_test_ext().execute_with(|| {
		// arrange
		let (test_account_2_key_pair, _) = sr25519::Pair::generate();
		let (payload, account_pk) = TestPasskeyPayloadBuilder::new()
			.with_a_valid_passkey()
			.with_passkey_as_payload()
			.with_call(RuntimeCall::Balances(BalancesCall::transfer_allow_death {
				dest: test_account_2_key_pair.public().into(),
				value: 10000,
			}))
			.with_funded_account(10000000000)
			.with_account_nonce(2)
			.build();

		let who: <Test as frame_system::Config>::AccountId = account_pk.into();
		let mut account = frame_system::Account::<Test>::get(&who);
		account.nonce += 1;
		frame_system::Account::<Test>::insert(who.clone(), account);

		// act
		let v = Passkey::validate_unsigned(TransactionSource::InBlock, &Call::proxy_v2 { payload });

		// assert
		assert!(v.is_ok());
		assert!(v.clone().unwrap().priority > 0);
		assert_eq!(v.clone().unwrap().requires, vec![Encode::encode(&(who.clone(), 1u64))]);
		assert_eq!(v.clone().unwrap().provides, vec![Encode::encode(&(who, 2u64))]);
	});
}

#[test]
fn validate_unsigned_with_exceeding_weights_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let block_length = BlockLength::default();
		let max = block_length.max.get(DispatchClass::Normal);
		let (payload, _) = TestPasskeyPayloadBuilder::new()
			.with_a_valid_passkey()
			.with_passkey_as_payload()
			.with_call(RuntimeCall::System(SystemCall::remark { remark: vec![1u8; *max as usize] }))
			.with_funded_account(10000000000)
			.build();

		// act
		let v = Passkey::validate_unsigned(TransactionSource::InBlock, &Call::proxy_v2 { payload });

		// assert
		assert_err!(v, InvalidTransaction::ExhaustsResources);
	});
}

#[test]
fn pre_dispatch_unsigned_with_used_nonce_should_fail_with_stale() {
	new_test_ext().execute_with(|| {
		// arrange
		let (test_account_2_key_pair, _) = sr25519::Pair::generate();
		let (payload, account_pk) = TestPasskeyPayloadBuilder::new()
			.with_a_valid_passkey()
			.with_passkey_as_payload()
			.with_call(RuntimeCall::Balances(BalancesCall::transfer_allow_death {
				dest: test_account_2_key_pair.public().into(),
				value: 10000,
			}))
			.with_funded_account(10000000000)
			.with_account_nonce(0)
			.build();
		let who: <Test as frame_system::Config>::AccountId = account_pk.into();
		let mut account = frame_system::Account::<Test>::get(&who);
		account.nonce += 1;
		frame_system::Account::<Test>::insert(who, account);

		// act
		let v = Passkey::pre_dispatch(&Call::proxy_v2 { payload });

		// assert
		assert_err!(v, InvalidTransaction::Stale);
	});
}

#[test]
fn pre_dispatch_unsigned_with_future_nonce_should_fail_with_future() {
	new_test_ext().execute_with(|| {
		// arrange
		let (test_account_2_key_pair, _) = sr25519::Pair::generate();
		let (payload, _) = TestPasskeyPayloadBuilder::new()
			.with_a_valid_passkey()
			.with_passkey_as_payload()
			.with_call(RuntimeCall::Balances(BalancesCall::transfer_allow_death {
				dest: test_account_2_key_pair.public().into(),
				value: 10000,
			}))
			.with_funded_account(10000000000)
			// setting a future nonce
			.with_account_nonce(2)
			.build();

		// act
		let v = Passkey::pre_dispatch(&Call::proxy_v2 { payload });

		// assert
		assert_err!(v, InvalidTransaction::Future);
	});
}

#[test]
fn pre_dispatch_unsigned_should_increment_nonce_on_success() {
	new_test_ext().execute_with(|| {
		// arrange
		let (payload, account_pk) = TestPasskeyPayloadBuilder::new()
			.with_a_valid_passkey()
			.with_passkey_as_payload()
			.with_call(RuntimeCall::System(SystemCall::remark { remark: vec![1u8; 3usize] }))
			.with_funded_account(10000000000)
			.build();
		let account_1_pk: <Test as frame_system::Config>::AccountId = account_pk.into();

		// act
		assert_ok!(Passkey::pre_dispatch(&Call::proxy_v2 { payload }));

		// assert
		let account = frame_system::Account::<Test>::get(&account_1_pk);
		assert_eq!(account.nonce, <Test as frame_system::Config>::Nonce::one());
	});
}

#[test]
fn pre_dispatch_with_exceeding_weight_should_fail() {
	new_test_ext().execute_with(|| {
		// arrange
		let block_length = BlockLength::default();
		let max = block_length.max.get(DispatchClass::Normal);
		let (payload, _) = TestPasskeyPayloadBuilder::new()
			.with_a_valid_passkey()
			.with_passkey_as_payload()
			.with_call(RuntimeCall::System(SystemCall::remark { remark: vec![1u8; *max as usize] }))
			.with_funded_account(10000000000)
			.build();

		// act
		let v = Passkey::pre_dispatch(&Call::proxy_v2 { payload });

		// assert
		assert_err!(v, InvalidTransaction::ExhaustsResources);
	});
}

#[test]
fn passkey_example_should_work() {
	new_test_ext().execute_with(|| {
		// arrange
		let account_id = AccountId32::new(from_hex("0x000000000000000000000000cf613044ccd8c1c60f561b99bd1fd2daef89625f").unwrap().try_into().unwrap());
		let pass_key_public_key = PasskeyPublicKey(from_hex("0x029bd263885e5eeaea31fa3b2e78ab1106d2cb1995045777fca3b38913a755d250").unwrap().try_into().unwrap());
		let payload = PasskeyPayloadV2 {
			passkey_public_key: pass_key_public_key,
			verifiable_passkey_signature: VerifiablePasskeySignature {
				signature: from_hex("0x30440220498c9d752c98f4cec77bb386f48dc52ccc6523f386c8942b3d50ff9dbdb4046a02205d2f2e3fa8bf27fd5c8f0d7f1ae8724a30dab1dac758019459114a447357c54b").unwrap().try_into().unwrap(),
				authenticator_data: from_hex("0x49960de5880e8c687434170f6476605b8fe4aeb9a28632c7995cf3ba831d97630500000003").unwrap().try_into().unwrap(),
				client_data_json: from_hex("0x7b2274797065223a22776562617574686e2e676574222c226368616c6c656e6765223a227372576f2d4a49393564336332393362774e756469747a53687a4f636849577a584d336478376d554f4351222c226f726967696e223a22687474703a2f2f6c6f63616c686f73743a3536353135222c2263726f73734f726967696e223a66616c73657d").unwrap().try_into().unwrap(),
			},
			account_ownership_proof: MultiSignature::Ecdsa(ecdsa::Signature::from_raw(from_hex("0xa0d9b3fe775b4be9d2b52f37f8a21c30be037a858f85cca82bce58ac05a95d9621063978dc086a83532e839882e701f2b7134541a4814526c888b8650c3eb5f81b").unwrap().try_into().unwrap())),
			passkey_call: PasskeyCallV2 {
				account_id: account_id.clone(),
				account_nonce: 0,
				call: Box::new(RuntimeCall::System(SystemCall::remark { remark: vec![1, 2, 3u8] })),
			},
		};
		assert_ok!(Balances::force_set_balance(
			RawOrigin::Root.into(),
			account_id,
			10000000000
		));

		// act
		let res =
			Passkey::validate_unsigned(TransactionSource::InBlock, &Call::proxy_v2 { payload });

		// assert
		assert!(res.is_ok());
	});
}
