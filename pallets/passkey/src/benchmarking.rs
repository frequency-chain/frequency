#![allow(clippy::unwrap_used)]
use super::*;

#[allow(unused)]
use crate::Pallet as Passkey;
use frame_benchmarking::{benchmarks, whitelisted_caller};

benchmarks! {
	proxy {
		let caller: T::AccountId = whitelisted_caller();
	}: {
		//TODO: should calculate overhead after applying all validations
	}
	verify {
	}

	impl_benchmark_test_suite!(
		Passkey,
		crate::mock::new_test_ext(),
		crate::mock::Test
	);
}
