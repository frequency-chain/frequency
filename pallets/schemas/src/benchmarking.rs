#![allow(clippy::unwrap_used)]
use common_primitives::schema::SchemaVersion;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::{assert_ok, ensure, BoundedVec};
use frame_system::RawOrigin;
use numtoa::NumToA;
use sp_std::vec;

use crate::Pallet as SchemasPallet;

use super::*;

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

benchmarks! {
	create_schema {
		let m in (T::MinSchemaModelSizeBytes::get() + 8) .. (T::SchemaModelMaxBytesBoundedVecLimit::get() - 1);
		let sender: T::AccountId = whitelisted_caller();
		let model_type = ModelType::AvroBinary;
		let payload_location = PayloadLocation::OnChain;
		assert_ok!(SchemasPallet::<T>::set_max_schema_model_bytes(RawOrigin::Root.into(), T::SchemaModelMaxBytesBoundedVecLimit::get()));
		let schema_input = generate_schema::<T>(m as usize);
	}: _(RawOrigin::Signed(sender), schema_input, model_type, payload_location)
	verify {
		ensure!(SchemasPallet::<T>::get_current_schema_identifier_maximum() > 0, "Created schema count should be > 0");
		ensure!(SchemasPallet::<T>::get_schema_info(1).is_some(), "Created schema should exist");
	}

	create_schema_via_governance {
		let m in (T::MinSchemaModelSizeBytes::get() + 8) .. (T::SchemaModelMaxBytesBoundedVecLimit::get() - 1);
		let sender: T::AccountId = whitelisted_caller();
		let model_type = ModelType::AvroBinary;
		let payload_location = PayloadLocation::OnChain;
		assert_ok!(SchemasPallet::<T>::set_max_schema_model_bytes(RawOrigin::Root.into(), T::SchemaModelMaxBytesBoundedVecLimit::get()));
		let schema_input = generate_schema::<T>(m as usize);
	}: _(RawOrigin::Root, sender.clone(), schema_input, model_type, payload_location, BoundedVec::default())
	verify {
		ensure!(SchemasPallet::<T>::get_current_schema_identifier_maximum() > 0, "Created schema count should be > 0");
		ensure!(SchemasPallet::<T>::get_schema_info(1).is_some(), "Created schema should exist");
	}

	propose_to_create_schema {
		let m in (T::MinSchemaModelSizeBytes::get() + 8) .. (T::SchemaModelMaxBytesBoundedVecLimit::get() - 1);
		let sender: T::AccountId = whitelisted_caller();
		let model_type = ModelType::AvroBinary;
		let payload_location = PayloadLocation::OnChain;
		assert_ok!(SchemasPallet::<T>::set_max_schema_model_bytes(RawOrigin::Root.into(), T::SchemaModelMaxBytesBoundedVecLimit::get()));
		let schema_input = generate_schema::<T>(m as usize);
	}: _(RawOrigin::Signed(sender), schema_input, model_type, payload_location, BoundedVec::default())
	verify {
		assert_eq!(T::ProposalProvider::proposal_count(), 1);
	}

	create_schema_v2 {
		let m in (T::MinSchemaModelSizeBytes::get() + 8) .. (T::SchemaModelMaxBytesBoundedVecLimit::get() - 1);
		let sender: T::AccountId = whitelisted_caller();
		let model_type = ModelType::AvroBinary;
		let payload_location = PayloadLocation::OnChain;
		assert_ok!(SchemasPallet::<T>::set_max_schema_model_bytes(RawOrigin::Root.into(), T::SchemaModelMaxBytesBoundedVecLimit::get()));
		let schema_input = generate_schema::<T>(m as usize);
	}: _(RawOrigin::Signed(sender), schema_input, model_type, payload_location, BoundedVec::default())
	verify {
		ensure!(SchemasPallet::<T>::get_current_schema_identifier_maximum() > 0, "Created schema count should be > 0");
		ensure!(SchemasPallet::<T>::get_schema_info(1).is_some(), "Created schema should exist");
	}

	create_schema_v3 {
		let m in (T::MinSchemaModelSizeBytes::get() + 8) .. (T::SchemaModelMaxBytesBoundedVecLimit::get() - 1);
		let sender: T::AccountId = whitelisted_caller();
		let version: SchemaVersion = 1;
		let namespace  = vec![b'a'; NAMESPACE_MAX as usize];
		let descriptor  = vec![b'b'; DESCRIPTOR_MAX as usize];
		let name:Vec<u8>= namespace.into_iter().chain(vec![b'.'].into_iter()).chain(descriptor.into_iter()).collect();
		let bounded_name = BoundedVec::try_from(name).expect("should resolve");
		let model_type = ModelType::AvroBinary;
		let payload_location = PayloadLocation::OnChain;
		assert_ok!(SchemasPallet::<T>::set_max_schema_model_bytes(RawOrigin::Root.into(), T::SchemaModelMaxBytesBoundedVecLimit::get()));
		let schema_input = generate_schema::<T>(m as usize);
	}: _(RawOrigin::Signed(sender), schema_input, model_type, payload_location, BoundedVec::default(), Some(bounded_name))
	verify {
		ensure!(SchemasPallet::<T>::get_current_schema_identifier_maximum() > 0, "Created schema count should be > 0");
		ensure!(SchemasPallet::<T>::get_schema_info(1).is_some(), "Created schema should exist");
	}

	set_max_schema_model_bytes {
		let sender = RawOrigin::Root;
		let max_size = T::SchemaModelMaxBytesBoundedVecLimit::get();
	}: _(sender, max_size)
	verify {
		ensure!(SchemasPallet::<T>::get_schema_model_max_bytes() == T::SchemaModelMaxBytesBoundedVecLimit::get(), "Schema model max should be updated!");
	}



	impl_benchmark_test_suite!(
		SchemasPallet,
		crate::tests::mock::new_test_ext(),
		crate::tests::mock::Test
	);
}
