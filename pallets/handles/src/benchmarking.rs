#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::RawOrigin;
use sp_core::crypto::KeyTypeId;
use sp_runtime::RuntimeAppPublic;

use crate::Pallet as Handles;

pub const TEST_KEY_TYPE_ID: KeyTypeId = KeyTypeId(*b"test");

mod app_sr25519 {
	use super::TEST_KEY_TYPE_ID;
	use sp_core::sr25519;
	use sp_runtime::app_crypto::app_crypto;
	app_crypto!(sr25519, TEST_KEY_TYPE_ID);
}

type SignerId = app_sr25519::Public;

fn create_signed_claims_payload<T: Config>(
	char_size: u32,
	byte_size: u32,
) -> (ClaimHandlePayload, MultiSignature, T::AccountId) {
	let delegator_account_public = SignerId::generate_pair(None);
	// create a generic handle example with expanding size
	let base_handle = b"b".to_vec();
	let mut handle = base_handle.clone();
	for _ in 0..char_size {
		if handle.len() >= byte_size as usize {
			break
		}
		handle.append(&mut base_handle.clone());
	}
	let handle_claims_payload = ClaimHandlePayload::new(handle);
	let encode_handle_claims_data = wrap_binary_data(handle_claims_payload.encode());
	let acc = T::AccountId::decode(&mut &delegator_account_public.encode()[..]).unwrap();

	let signature = delegator_account_public.sign(&encode_handle_claims_data).unwrap();
	(handle_claims_payload, MultiSignature::Sr25519(signature.into()), acc.into())
}

benchmarks! {
	claim_handle {
		// create a generic handle example with expanding size 3 to 32 bytes
		let c in HANDLE_BASE_CHARS_MIN .. HANDLE_BASE_CHARS_MAX-1;
		let b in HANDLE_BASE_BYTES_MIN .. HANDLE_BASE_BYTES_MAX-1;
		let caller: T::AccountId = whitelisted_caller();
		let delegator_msa_id = 1u64;
		let (payload, proof, key) = create_signed_claims_payload::<T>(c, b);
		assert_ok!(T::MsaBenchmarkHelper::add_key(delegator_msa_id.into(), caller.clone()));
		assert_ok!(T::MsaBenchmarkHelper::add_key(delegator_msa_id.into(), key.clone()));

	}: _(RawOrigin::Signed(caller.clone()), key.clone(), proof, payload)
	verify {
		let stored_handle = Handles::<T>::get_handle_for_msa(delegator_msa_id.into());
		assert!(stored_handle.is_some());
	}

	impl_benchmark_test_suite!(Handles, crate::mock::new_tester(), crate::mock::Test,);
}
