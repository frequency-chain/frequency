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
use common_primitives::signatures::{UnifiedSignature, UnifiedSigner};
use pallet_balances::Call as BalancesCall;
use sp_core::{bytes::from_hex, ecdsa, sr25519, sr25519::Public, Pair};
use sp_runtime::{
	traits::{IdentifyAccount, One, Verify},
	DispatchError::BadOrigin,
};

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

	pub fn build(&self) -> (PasskeyPayload<Test>, Public) {
		let wrapped_binary = wrap_binary_data(self.payload_to_sign.clone());
		let signature: MultiSignature = self.key_pair.sign(wrapped_binary.as_slice()).into();
		let client_data = base64_url::decode(REPLACED_CLIENT_DATA_JSON).unwrap();
		let authenticator = base64_url::decode(AUTHENTICATOR_DATA).unwrap();
		let bad_authenticator = b"bad_auth".to_vec();
		let call: PasskeyCall<Test> = PasskeyCall {
			account_id: self.key_pair.public().into(),
			account_nonce: self.nonce.into(),
			account_ownership_proof: signature,
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
		let payload = PasskeyPayload {
			passkey_public_key: self.passkey_public_key.clone(),
			verifiable_passkey_signature: VerifiablePasskeySignature {
				signature: passkey_signature,
				client_data_json: client_data.try_into().unwrap(),
				authenticator_data: authenticator.try_into().unwrap(),
			},
			passkey_call: call,
		};
		(payload, self.key_pair.public())
	}
}

#[test]
#[allow(deprecated)]
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
		assert_noop!(Passkey::proxy(RuntimeOrigin::signed(account_pk.into()), payload), BadOrigin);
	});
}

#[test]
#[allow(deprecated)]
fn proxy_call_with_unsigned_origin_should_work() {
	new_test_ext().execute_with(|| {
		// arrange
		let (payload, _) = TestPasskeyPayloadBuilder::new()
			.with_passkey_as_payload()
			.with_call(RuntimeCall::System(SystemCall::remark { remark: vec![1, 2, 3u8] }))
			.build();

		// assert
		assert_ok!(Passkey::proxy(RuntimeOrigin::none(), payload));
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

		let res = Passkey::validate_unsigned(TransactionSource::InBlock, &Call::proxy { payload });
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

		let res = Passkey::validate_unsigned(TransactionSource::InBlock, &Call::proxy { payload });
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
		let res = Passkey::validate_unsigned(TransactionSource::InBlock, &Call::proxy { payload });
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
		let res = Passkey::validate_unsigned(TransactionSource::InBlock, &Call::proxy { payload });

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
		let res = Passkey::pre_dispatch(&Call::proxy { payload });

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
		let res = Passkey::pre_dispatch(&Call::proxy { payload });

		// assert
		assert_err!(res, InvalidTransaction::Payment);
	});
}

#[test]
fn validate_unsigned_should_not_remove_fee_on_successful_validation() {
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
		let res = Passkey::validate_unsigned(TransactionSource::InBlock, &Call::proxy { payload });

		// assert
		assert!(res.is_ok());
		let final_balance = Balances::free_balance(&account_id);
		assert_eq!(final_balance, initial_balance);
	});
}

#[test]
fn pre_dispatch_should_remove_fee_on_successful_validation() {
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
		let res = Passkey::pre_dispatch(&Call::proxy { payload });

		// assert
		assert!(res.is_ok());
		let final_balance = Balances::free_balance(&account_id);
		assert!(final_balance < initial_balance);
	});
}

#[test]
#[allow(deprecated)]
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
		let validate_result = Passkey::pre_dispatch(&Call::proxy { payload: payload.clone() });
		let extrinsic_result = Passkey::proxy(RuntimeOrigin::none(), payload);

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
		let v = Passkey::validate_unsigned(TransactionSource::InBlock, &Call::proxy { payload });

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
		let v = Passkey::validate_unsigned(TransactionSource::InBlock, &Call::proxy { payload });

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
		let v = Passkey::validate_unsigned(TransactionSource::InBlock, &Call::proxy { payload });

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
		let v = Passkey::validate_unsigned(TransactionSource::InBlock, &Call::proxy { payload });

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
		let v = Passkey::pre_dispatch(&Call::proxy { payload });

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
		let v = Passkey::pre_dispatch(&Call::proxy { payload });

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
		assert_ok!(Passkey::pre_dispatch(&Call::proxy { payload }));

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
		let v = Passkey::pre_dispatch(&Call::proxy { payload });

		// assert
		assert_err!(v, InvalidTransaction::ExhaustsResources);
	});
}

#[test]
fn passkey_public_key_scale_and_eip712_compatibility_guard() {
	let public_key = [
		0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
		25, 26, 27, 28, 29, 30, 31, 32,
	];
	let pass_key = PasskeyPublicKey(public_key.clone());

	assert_eq!(pass_key.encode(), pass_key.inner().encode());
	assert_eq!(pass_key.encode(), pass_key.inner());
	assert_eq!(pass_key.encode(), public_key);
}

#[test]
fn ethereum_eip712_signatures_for_passkey_publickey_should_work() {
	new_test_ext().execute_with(|| {
		let payload = PasskeyPublicKey (from_hex("0x40a6836ea489047852d3f0297f8fe8ad6779793af4e9c6274c230c207b9b825026").unwrap().try_into().unwrap());
		let encoded_payload = payload.encode_eip_712(420420420u32);

		// following signature is generated via Metamask using the same input to check compatibility
		let signature_raw = from_hex("0xbafaf5e21695a502b2d356b4558da35245aa1be7161f01a5f0224fbfdf85b5c52898fc495ab1ca9b68c3b07e23d31a5fe1686165344b22bc14201f293d54f36b1b").expect("Should convert");
		let unified_signature = UnifiedSignature::from(ecdsa::Signature::from_raw(
			signature_raw.try_into().expect("should convert"),
		));

		// Non-compressed public key associated with the keypair used in Metamask
		// 0x509540919faacf9ab52146c9aa40db68172d83777250b28e4679176e49ccdd9fa213197dc0666e85529d6c9dda579c1295d61c417f01505765481e89a4016f02
		let public_key = ecdsa::Public::from_raw(
			from_hex("0x02509540919faacf9ab52146c9aa40db68172d83777250b28e4679176e49ccdd9f")
				.expect("should convert")
				.try_into()
				.expect("invalid size"),
		);
		let unified_signer = UnifiedSigner::from(public_key);
		assert!(unified_signature.verify(&encoded_payload[..], &unified_signer.into_account()));
	});
}
