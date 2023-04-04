#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as Handles;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::RawOrigin;
use scale_info::prelude::format;
use sp_core::crypto::KeyTypeId;
use sp_runtime::RuntimeAppPublic;

pub const TEST_KEY_TYPE_ID: KeyTypeId = KeyTypeId(*b"test");

mod app_sr25519 {
	use super::TEST_KEY_TYPE_ID;
	use sp_core::sr25519;
	use sp_runtime::app_crypto::app_crypto;
	app_crypto!(sr25519, TEST_KEY_TYPE_ID);
}

type SignerId = app_sr25519::Public;

fn create_signed_claims_payload<T: Config>(
	delegator_account_public: SignerId,
	byte_size: u32,
) -> (ClaimHandlePayload, MultiSignature, T::AccountId) {
	// create a generic handle example with expanding size
	let base_handle = b"b".to_vec();
	let max_chars = 20;

	// calculate maximum byte size based on maximum number of allowed characters
	// 32
	let max_32_bytes = max_chars * 3 / 2;

	// limit byte size to a maximum of 80 bytes
	let byte_size = byte_size.min(max_32_bytes);

	// create handle with limited number of characters
	let mut handle = base_handle.clone();
	handle.resize(byte_size as usize, b'b');
	let handle_str = core::str::from_utf8(&handle).unwrap_or_default();
	let truncated_handle: Vec<u8> = handle_str
		.chars()
		.take(max_chars as usize)
		.flat_map(|c| c.encode_utf8(&mut [0; 4]).as_bytes().to_vec())
		.collect();
	let handle_claims_payload = ClaimHandlePayload::new(truncated_handle);
	let encode_handle_claims_data = wrap_binary_data(handle_claims_payload.encode());
	let acc = T::AccountId::decode(&mut &delegator_account_public.encode()[..]).unwrap();

	let signature = delegator_account_public.sign(&encode_handle_claims_data).unwrap();
	(handle_claims_payload, MultiSignature::Sr25519(signature.into()), acc.into())
}

benchmarks! {
	claim_handle {
		// claim a handle
		let b in HANDLE_BASE_BYTES_MIN .. HANDLE_BASE_BYTES_MAX-2;
		let caller: T::AccountId = whitelisted_caller();
		let delegator_msa_id = 1u64;
		let delegator_account_public = SignerId::generate_pair(None);
		let (payload, proof, key) = create_signed_claims_payload::<T>(delegator_account_public, b);
		assert_ok!(T::MsaBenchmarkHelper::add_key(delegator_msa_id.into(), caller.clone()));
		assert_ok!(T::MsaBenchmarkHelper::add_key(delegator_msa_id.into(), key.clone()));

	}: _(RawOrigin::Signed(caller.clone()), key.clone(), proof, payload)
	verify {
		let stored_handle = Handles::<T>::get_handle_for_msa(delegator_msa_id.into());
		assert!(stored_handle.is_some());
	}

	retire_handle {
		// claim a handle
		let b in HANDLE_BASE_BYTES_MIN .. HANDLE_BASE_BYTES_MAX-1;
		let caller: T::AccountId = whitelisted_caller();
		let delegator_msa_id = 1u64;
		let delegator_account_public = SignerId::generate_pair(None);
		let (payload, proof, key) = create_signed_claims_payload::<T>(delegator_account_public.clone(), b);
		assert_ok!(T::MsaBenchmarkHelper::add_key(delegator_msa_id.into(), caller.clone()));
		assert_ok!(T::MsaBenchmarkHelper::add_key(delegator_msa_id.into(), key.clone()));
		assert_ok!(Handles::<T>::claim_handle(RawOrigin::Signed(caller.clone()).into(), key.clone(), proof, payload));
		let stored_handle = Handles::<T>::get_handle_for_msa(delegator_msa_id.into());
		assert!(stored_handle.is_some());

		// retire the handle
		let stored_handle = stored_handle.unwrap();
		let base_handle:Vec<u8> = stored_handle.base_handle.clone();
		let suffix: u16 = stored_handle.suffix;
		let base_handle_str = core::str::from_utf8(&base_handle).unwrap_or_default();
		let full_handle_with_delimiter = format!("{}{}", base_handle_str, ".");
		let retirement_payload = RetireHandlePayload::new(full_handle_with_delimiter.as_bytes().to_vec());
		let encode_handle_claims_data = wrap_binary_data(retirement_payload.encode());
		let signature = delegator_account_public.sign(&encode_handle_claims_data).unwrap();
		let retire_proof = MultiSignature::Sr25519(signature.into());
	}: _(RawOrigin::Signed(caller.clone()), key.clone(), retire_proof, retirement_payload)
	verify {
		let stored_handle = Handles::<T>::get_handle_for_msa(delegator_msa_id.into());
		assert!(stored_handle.is_none());
	}
	impl_benchmark_test_suite!(Handles, crate::tests::mock::new_test_ext_keystore(), crate::tests::mock::Test,);
}
