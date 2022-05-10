use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::{assert_ok, ensure};
use frame_system::RawOrigin;
use sp_std::vec::Vec;

use crate::Pallet as SchemasPallet;

use super::*;

const SCHEMAS: u32 = 5000;

const EXAMPLE_SCHEMA: &[u8] = r#"{"type":"record","name":"mrc.schema.root",
	"fields": [{"name": "name","type": "string"},
    {"name": "type","type": "string"}}"#
	.as_bytes();

fn generate_schema(size: usize) -> Vec<u8> {
	Vec::from(&EXAMPLE_SCHEMA[..size])
}

fn register_some_schema<T: Config>(sender: T::AccountId) -> DispatchResult {
	let schema_size: usize = (T::MaxSchemaSizeBytes::get() - 1) as usize;
	SchemasPallet::<T>::register_schema(
		RawOrigin::Signed(sender).into(),
		generate_schema(schema_size),
	)
}

benchmarks! {
	register_schema {
		let m in (T::MinSchemaSizeBytes::get()) .. (T::MaxSchemaSizeBytes::get());
		let n in 1 .. SCHEMAS;
		let sender: T::AccountId = whitelisted_caller();
		for j in 0..(n) {
			assert_ok!(register_some_schema::<T>(sender.clone()));
		}
	}: _(RawOrigin::Signed(sender), generate_schema(m as usize))

	verify {
		ensure!(SchemasPallet::<T>::schema_count() > 0, "Registered schema count should be > 0");
	}
	impl_benchmark_test_suite!(
		SchemasPallet,
		crate::mock::new_test_ext(),
		crate::mock::Test
	);
}
