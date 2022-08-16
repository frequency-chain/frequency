use frame_benchmarking::{benchmarks, vec, whitelisted_caller};
use frame_support::{assert_ok, ensure, BoundedVec};
use frame_system::RawOrigin;
use numtoa::NumToA;
use sp_std::vec::Vec;

use crate::Pallet as SchemasPallet;

use super::*;

const SCHEMAS: u32 = 1000;

fn generate_schema<T: Config>(
	size: usize,
) -> BoundedVec<u8, T::SchemaModelMaxBytesBoundedVecLimit> {
	let mut json: Vec<u8> = vec![];
	json.extend(b"{");
	for i in 0..size {
		let mut item: Vec<u8> = vec![];
		item.extend(b"\"key");
		let mut buff = [0u8; 30];
		item.extend(i.numtoa(10, &mut buff));
		item.extend(b"\":\"val\",");
		if item.len() + json.len() < size {
			json.extend(item);
		} else {
			break
		}
	}
	json.pop(); // removing last ,
	json.extend(b"}");
	json.try_into().unwrap()
}

fn register_some_schema<T: Config>(
	sender: T::AccountId,
	model_type: ModelType,
	payload_location: PayloadLocation,
) -> DispatchResult {
	let schema_size: usize = (T::SchemaModelMaxBytesBoundedVecLimit::get() / 2) as usize;
	SchemasPallet::<T>::register_schema(
		RawOrigin::Signed(sender).into(),
		generate_schema::<T>(schema_size),
		model_type,
		payload_location,
	)
}

benchmarks! {
	register_schema {
		let m in (T::MinSchemaModelSizeBytes::get() + 8) .. (T::SchemaModelMaxBytesBoundedVecLimit::get() - 1);
		let n in 1 .. SCHEMAS;
		let sender: T::AccountId = whitelisted_caller();
		let model_type = ModelType::default();
		let payload_location = PayloadLocation::default();
		assert_ok!(SchemasPallet::<T>::set_max_schema_model_bytes(RawOrigin::Root.into(), T::SchemaModelMaxBytesBoundedVecLimit::get()));
		for j in 0..(n) {
			assert_ok!(register_some_schema::<T>(sender.clone(), model_type.clone(), payload_location.clone()));
		}
		let schema_input = generate_schema::<T>(m as usize);
	}: _(RawOrigin::Signed(sender), schema_input, model_type, payload_location)
	verify {
		ensure!(SchemasPallet::<T>::schema_count() > 0, "Registered schema count should be > 0");
	}
	impl_benchmark_test_suite!(
		SchemasPallet,
		crate::mock::new_test_ext(),
		crate::mock::Test
	);
}
