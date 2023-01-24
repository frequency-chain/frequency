use super::*;
use crate::Pallet as StatefulMessageStorage;

use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
	add_item {
		let caller: T::AccountId = whitelisted_caller();
	}: _ (RawOrigin::Signed(caller.clone()))
	verify {
		assert_eq!(frame_system::Pallet::<T>::events().len(), 0);
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
