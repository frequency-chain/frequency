#![allow(clippy::unwrap_used)]
use super::*;

#[allow(unused)]
use crate::Pallet as Passkey;
use crate::{
	test_common::{
		constants::{AUTHENTICATOR_DATA, REPLACED_CLIENT_DATA_JSON},
		utilities::{get_p256_public_key, passkey_sign},
	},
	types::*,
};
use common_primitives::utils::wrap_binary_data;
use frame_benchmarking::benchmarks;
use frame_support::assert_ok;
use sp_core::{crypto::KeyTypeId, Encode};
use sp_runtime::{traits::Zero, MultiSignature, RuntimeAppPublic};
extern crate alloc;
use alloc::boxed::Box;

pub const TEST_KEY_TYPE_ID: KeyTypeId = KeyTypeId(*b"test");

mod app_sr25519 {
	use super::TEST_KEY_TYPE_ID;
	use sp_core::sr25519;
	use sp_runtime::app_crypto::app_crypto;
	app_crypto!(sr25519, TEST_KEY_TYPE_ID);
}

type SignerId = app_sr25519::Public;

fn generate_payload<T: Config>() -> PasskeyPayloadV2<T> {
	let test_account_1_pk = SignerId::generate_pair(None);
	let test_account_1_account_id =
		T::AccountId::decode(&mut &test_account_1_pk.encode()[..]).unwrap();
	T::Currency::set_balance(&test_account_1_account_id.clone().into(), 4_000_000_000u32.into());
	let secret = p256::SecretKey::from_slice(&[
		1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8, 1, 2, 3, 4, 5, 6, 7, 8,
	])
	.unwrap();
	let passkey_public_key = get_p256_public_key(&secret).unwrap();
	let wrapped_binary = wrap_binary_data(passkey_public_key.inner().to_vec());
	let signature: MultiSignature =
		MultiSignature::Sr25519(test_account_1_pk.sign(&wrapped_binary).unwrap().into());
	let client_data = base64_url::decode(REPLACED_CLIENT_DATA_JSON).unwrap();
	let authenticator = base64_url::decode(AUTHENTICATOR_DATA).unwrap();

	let inner_call: <T as Config>::RuntimeCall =
		frame_system::Call::<T>::remark { remark: vec![] }.into();

	let call: PasskeyCallV2<T> = PasskeyCallV2 {
		account_id: test_account_1_account_id,
		account_nonce: T::Nonce::zero(),
		call: Box::new(inner_call),
	};

	let passkey_signature =
		passkey_sign(&secret, &call.encode(), &client_data, &authenticator).unwrap();
	let payload = PasskeyPayloadV2 {
		passkey_public_key,
		verifiable_passkey_signature: VerifiablePasskeySignature {
			signature: passkey_signature,
			client_data_json: client_data.try_into().unwrap(),
			authenticator_data: authenticator.try_into().unwrap(),
		},
		account_ownership_proof: signature,
		passkey_call: call,
	};
	payload
}

benchmarks! {
	where_clause {  where
		BalanceOf<T>: From<u64>,
		<T as frame_system::Config>::RuntimeCall: From<Call<T>> + Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
		<T as frame_system::Config>::RuntimeOrigin: AsTransactionAuthorizedOrigin,
	}

	validate {
		let payload = generate_payload::<T>();
	}: {
		assert_ok!(Passkey::validate_unsigned(TransactionSource::InBlock, &Call::proxy_v2 { payload }));
	}

	pre_dispatch {
		let payload = generate_payload::<T>();
	}: {
		assert_ok!(Passkey::pre_dispatch(&Call::proxy_v2 { payload }));
	}

	impl_benchmark_test_suite!(
		Passkey,
		crate::mock::new_test_ext_keystore(),
		crate::mock::Test
	);
}
