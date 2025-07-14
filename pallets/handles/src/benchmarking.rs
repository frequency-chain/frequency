#![allow(clippy::unwrap_used)]

use super::*;
use crate::Pallet as Handles;
use common_primitives::utils::wrap_binary_data;
use frame_benchmarking::{v2::*, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::RawOrigin;
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
) -> (ClaimHandlePayload<BlockNumberFor<T>>, MultiSignature, T::AccountId, MessageSourceId) {
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
	let signature_expires_at: BlockNumberFor<T> = 10u32.into();
	let handle_claims_payload =
		ClaimHandlePayload::<BlockNumberFor<T>>::new(truncated_handle, signature_expires_at);
	let encode_handle_claims_data = wrap_binary_data(handle_claims_payload.encode());
	let acc = T::AccountId::decode(&mut &delegator_account_public.encode()[..]).unwrap();
	let msa_id = MessageSourceId::decode(&mut &delegator_account_public.encode()[..]).unwrap();

	let signature = delegator_account_public.sign(&encode_handle_claims_data).unwrap();
	(handle_claims_payload, MultiSignature::Sr25519(signature.into()), acc, msa_id)
}

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn claim_handle(
		b: Linear<HANDLE_BYTES_MIN, { HANDLE_BYTES_MAX - 2 }>,
	) -> Result<(), BenchmarkError> {
		// claim a handle
		let caller: T::AccountId = whitelisted_caller();
		let delegator_account_public = SignerId::generate_pair(None);
		let (payload, proof, key, delegator_msa_id) =
			create_signed_claims_payload::<T>(delegator_account_public, b);
		assert_ok!(T::MsaBenchmarkHelper::add_key(delegator_msa_id, caller.clone()));
		assert_ok!(T::MsaBenchmarkHelper::add_key(delegator_msa_id, key.clone()));

		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()), key.clone(), proof, payload);

		let stored_handle = Handles::<T>::get_handle_for_msa(delegator_msa_id);
		assert!(stored_handle.is_some());
		Ok(())
	}

	#[benchmark]
	fn change_handle(
		b: Linear<HANDLE_BYTES_MIN, { HANDLE_BYTES_MAX - 2 }>,
	) -> Result<(), BenchmarkError> {
		// claim a handle to be changed
		let caller: T::AccountId = whitelisted_caller();
		let delegator_account_public = SignerId::generate_pair(None);
		let (payload, proof, key, delegator_msa_id) =
			create_signed_claims_payload::<T>(delegator_account_public.clone(), b);
		assert_ok!(T::MsaBenchmarkHelper::add_key(delegator_msa_id, caller.clone()));
		assert_ok!(T::MsaBenchmarkHelper::add_key(delegator_msa_id, key.clone()));
		assert_ok!(Handles::<T>::claim_handle(
			RawOrigin::Signed(caller.clone()).into(),
			key.clone(),
			proof.clone(),
			payload.clone()
		));

		#[extrinsic_call]
		_(RawOrigin::Signed(caller.clone()), key.clone(), proof, payload);

		let stored_handle = Handles::<T>::get_handle_for_msa(delegator_msa_id);
		assert!(stored_handle.is_some());
		Ok(())
	}

	#[benchmark]
	fn retire_handle() -> Result<(), BenchmarkError> {
		// claim a handle to be retired
		let caller: T::AccountId = whitelisted_caller();
		let delegator_account_public = SignerId::generate_pair(None);
		let (payload, proof, key, delegator_msa_id) =
			create_signed_claims_payload::<T>(delegator_account_public.clone(), 32);
		assert_ok!(T::MsaBenchmarkHelper::add_key(delegator_msa_id, caller.clone()));
		assert_ok!(T::MsaBenchmarkHelper::add_key(delegator_msa_id, key.clone()));
		assert_ok!(Handles::<T>::claim_handle(
			RawOrigin::Signed(caller.clone()).into(),
			key.clone(),
			proof,
			payload
		));
		let stored_handle = Handles::<T>::get_handle_for_msa(delegator_msa_id);
		assert!(stored_handle.is_some());

		frame_system::Pallet::<T>::set_block_number(500u32.into());

		// retire the handle
		#[extrinsic_call]
		_(RawOrigin::Signed(key.clone()));

		let stored_handle = Handles::<T>::get_handle_for_msa(delegator_msa_id);
		assert!(stored_handle.is_none());
		Ok(())
	}
	impl_benchmark_test_suite!(
		Handles,
		crate::tests::mock::new_test_ext_keystore(),
		crate::tests::mock::Test,
	);
}
