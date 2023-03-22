//! Benchmarking setup for handles pallet
use super::*;
use frame_benchmarking::whitelisted_caller;

#[allow(unused)]
use crate::Pallet as Handles;
use common_primitives::handles::*;
use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;

benchmarks! {

		claim_handle {
			let s = HANDLE_BASE_MAX;
			let caller: T::AccountId = whitelisted_caller();
			let handle_name = (1 .. s as u8).collect::<Vec<_>>();
		}: _ (RawOrigin::Signed(caller.clone()), handle_name)
		verify {
		}

impl_benchmark_test_suite!(Handles,
	crate::tests::mock::new_test_ext(),
	crate::tests::mock::Test);
}
