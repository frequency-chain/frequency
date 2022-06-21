use frame_benchmarking::{benchmarks, vec, whitelisted_caller};
use frame_support::{assert_ok, ensure};
use frame_system::RawOrigin;

use crate::Pallet as SchemasPallet;

use super::*;

const SCHEMAS: u32 = 1000;

fn generate_schema<T: Config>(
	size: usize,
) -> BoundedVec<u8, T::SchemaModelMaxBytesBoundedVecLimit> {
	let input = vec![1; size as usize];
	input.try_into().unwrap()
}

fn register_some_schema<T: Config>(sender: T::AccountId) -> DispatchResult {
	let schema_size: usize = (T::SchemaModelMaxBytesBoundedVecLimit::get() / 2) as usize;
	SchemasPallet::<T>::register_schema(
		RawOrigin::Signed(sender).into(),
		generate_schema::<T>(schema_size),
	)
}

benchmarks! {
	register_schema {
		let m in (T::MinSchemaModelSizeBytes::get() + 1) .. (T::SchemaModelMaxBytesBoundedVecLimit::get() - 1);
		let n in 1 .. SCHEMAS;
		let sender: T::AccountId = whitelisted_caller();
		assert_ok!(SchemasPallet::<T>::set_max_schema_model_bytes(RawOrigin::Root.into(), T::SchemaModelMaxBytesBoundedVecLimit::get()));
		for j in 0..(n) {
			assert_ok!(register_some_schema::<T>(sender.clone()));
		}
	}: _(RawOrigin::Signed(sender), generate_schema::<T>(m as usize))

	verify {
		ensure!(SchemasPallet::<T>::schema_count() > 0, "Registered schema count should be > 0");
	}
	impl_benchmark_test_suite!(
		SchemasPallet,
		crate::mock::new_test_ext(),
		crate::mock::Test
	);
}
