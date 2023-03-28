//! Benchmarking setup for handles pallet
use super::*;
use frame_benchmarking::whitelisted_caller;

#[allow(unused)]
use crate::Pallet as Handles;
use common_primitives::{handles::*, utils::wrap_binary_data};
use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;
use sp_core::{sr25519, Encode, Pair};
use sp_runtime::MultiSignature;

benchmarks! {

		claim_handle {
			let s = HANDLE_BASE_CHARS_MAX;
			let provider: T::AccountId = whitelisted_caller();
			// Delegator
			let delegator_key_pair = sr25519::Pair::generate().0;
			let delegator_account = delegator_key_pair.public();
			let delegator: T::AccountId = whitelisted_caller();  // FIX THIS

			let base_handle = (1 .. s as u8).collect::<Vec<_>>();
			// Payload
			let payload = ClaimHandlePayload::new(base_handle.clone());
			let encoded_payload = wrap_binary_data(payload.encode());
			// Proof
			let proof: MultiSignature = delegator_key_pair.sign(&encoded_payload).into();
		}: _ (RawOrigin::Signed(provider.clone()), delegator, proof, payload)
		verify {
		}

impl_benchmark_test_suite!(Handles,
	crate::tests::mock::new_test_ext(),
	crate::tests::mock::Test);
}
