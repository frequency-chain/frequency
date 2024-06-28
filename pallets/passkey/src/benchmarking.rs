#![allow(clippy::unwrap_used)]
use super::*;

use crate::types::*;
#[allow(unused)]
use crate::Pallet as Passkey;
use frame_benchmarking::benchmarks;
use frame_support::assert_ok;
use sp_core::{crypto::KeyTypeId, Encode};
use sp_runtime::{traits::Zero, MultiSignature, RuntimeAppPublic};
use sp_std::prelude::*;

pub const TEST_KEY_TYPE_ID: KeyTypeId = KeyTypeId(*b"test");

mod app_sr25519 {
	use super::TEST_KEY_TYPE_ID;
	use sp_core::sr25519;
	use sp_runtime::app_crypto::app_crypto;
	app_crypto!(sr25519, TEST_KEY_TYPE_ID);
}

type SignerId = app_sr25519::Public;
const REPLACED_CLIENT_DATA_JSON: &'static str = "eyJ0eXBlIjoid2ViYXV0aG4uZ2V0IiwiY2hhbGxlbmdlIjoiI3JwbGMjIiwib3JpZ2luIjoiaHR0cHM6Ly9wYXNza2V5LmFtcGxpY2EuaW86ODA4MCIsImNyb3NzT3JpZ2luIjpmYWxzZSwiYWxnIjoiSFMyNTYifQ";
const AUTHENTICATOR_DATA: &'static str = "WJ8JTNbivTWn-433ubs148A7EgWowi4SAcYBjLWfo1EdAAAAAA";

fn generate_payload<T: Config>() -> PasskeyPayload<T> {
	let test_account_1_pk = SignerId::generate_pair(None);
	let passkey_public_key = PasskeyPublicKey([0u8; 33]);
	let wrapped_binary = wrap_binary_data(passkey_public_key.inner().to_vec());
	let signature: MultiSignature =
		MultiSignature::Sr25519(test_account_1_pk.sign(&wrapped_binary).unwrap().into());

	let inner_call: <T as Config>::RuntimeCall =
		frame_system::Call::<T>::remark { remark: vec![] }.into();

	let call: PasskeyCall<T> = PasskeyCall {
		account_id: T::AccountId::decode(&mut &test_account_1_pk.encode()[..]).unwrap(),
		account_nonce: T::Nonce::zero(),
		account_ownership_proof: signature,
		call: Box::new(inner_call),
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
	payload
}

benchmarks! {
	where_clause {  where <T as frame_system::Config>::RuntimeCall: Dispatchable<Info = DispatchInfo> }

	validate {
		let payload = generate_payload::<T>();
	}: {
		assert_ok!(Passkey::validate_unsigned(TransactionSource::InBlock, &Call::proxy { payload }));
	}

	pre_dispatch {
		let payload = generate_payload::<T>();
	}: {
		assert_ok!(Passkey::pre_dispatch(&Call::proxy { payload }));
	}

	impl_benchmark_test_suite!(
		Passkey,
		crate::mock::new_test_ext_keystore(),
		crate::mock::Test
	);
}
