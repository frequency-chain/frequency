use super::*;

#[allow(unused)]
use crate::Pallet as Msa;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::RawOrigin;

const SEED: u32 = 0;

fn create_account<T: Config>(name: &'static str, index: u32) -> T::AccountId {
	account(name, index, SEED)
}

fn create_msa<T: Config>(n: u32) -> DispatchResult {
	let acc = create_account::<T>("account", n);
	Msa::<T>::create(RawOrigin::Signed(acc.clone()).into())
}

benchmarks! {
	create {
		let s in 1 .. 1000;
		let caller: T::AccountId = whitelisted_caller();

		for j in 0 .. s {
			assert_ok!(create_msa::<T>(j));
		}
	}: _ (RawOrigin::Signed(caller))

	impl_benchmark_test_suite!(Msa, crate::mock::new_test_ext(), crate::mock::Test);
}
