use super::*;
use crate::Pallet as StatefulMessageStorage;

use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
	add_item {
		let n in 0 .. 5;
		let caller: T::AccountId = whitelisted_caller();

		let payload = vec![1; n as usize];
	}: _ (RawOrigin::Signed(caller), payload)
	verify {
		assert_eq!(false, false);
	}

	remove_item {
		let caller: T::AccountId = whitelisted_caller();
	}: _ (RawOrigin::Signed(caller.clone()))
	verify {
	}

	upsert_page {
		let caller: T::AccountId = whitelisted_caller();
	}: _ (RawOrigin::Signed(caller.clone()))
	verify {
	}

	remove_page {
		let caller: T::AccountId = whitelisted_caller();
	}: _ (RawOrigin::Signed(caller.clone()))
	verify {
	}

	impl_benchmark_test_suite!(StatefulMessageStorage,
		crate::mock::new_test_ext(),
		crate::mock::Test);
}
