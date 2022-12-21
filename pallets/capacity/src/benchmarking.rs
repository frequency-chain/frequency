use super::*;
use crate::Pallet as Capacity;

use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
	stake {
		let caller: T::AccountId = whitelisted_caller();
		let amount: BalanceOf<T> = 1u32.into();
		let target = 1;

	}: _ (RawOrigin::Signed(caller.clone()), target, amount)
	verify {
	}

	impl_benchmark_test_suite!(Capacity,
		crate::mock::new_test_ext(),
		crate::mock::Test);
}
