#![allow(clippy::unwrap_used)]
use super::*;

#[allow(unused)]
use crate::Pallet as Passkey;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use sp_std::prelude::*;

benchmarks! {
	proxy {
		let caller: T::AccountId = whitelisted_caller();
	}: _(RawOrigin::Signed(caller))
	verify {
	}

	impl_benchmark_test_suite!(
		Passkey,
		crate::mock::new_test_ext(),
		crate::mock::Test
	);
}
