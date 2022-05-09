use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::{assert_ok, ensure};
use frame_system::RawOrigin;
use sp_std::vec::Vec;

use crate::Pallet as SchemasPallet;

use super::*;

fn random_schema() -> Vec<u8> {
	Vec::from("aasdf,sfdfb,eeec,dfsdfs,egsdkld,sldfklzz,sffs,sdjkl,lslsls,lfkjsf".as_bytes())
}

fn register_random_schema<T: Config>(sender: T::AccountId) -> DispatchResult {
	SchemasPallet::<T>::register_schema(RawOrigin::Signed(sender).into(), random_schema())
}

benchmarks! {
	register_schema {
		let n in 1..(T::MaxSchemaRegistrations::get() - 1).into();
		let sender: T::AccountId = whitelisted_caller();
		for j in 0..(n) {
			assert_ok!(register_random_schema::<T>(sender.clone()));
		}
	}: _(RawOrigin::Signed(sender), random_schema())

	verify {
		ensure!(SchemasPallet::<T>::schema_count() > 0, "Registered schema count should be > 0");
	}
	impl_benchmark_test_suite!(
		SchemasPallet,
		crate::mock::new_test_ext(),
		crate::mock::Test
	);
}
