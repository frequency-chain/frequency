#![allow(clippy::unwrap_used)]
use frame_benchmarking::{v2::*, whitelisted_caller};
use frame_support::{assert_ok, ensure, BoundedVec};
use frame_system::RawOrigin;
use numtoa::NumToA;
extern crate alloc;
use alloc::vec;

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

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn create_schema_v3_with_name(
		m: Linear<
			{ T::MinSchemaModelSizeBytes::get() + 8 },
			{ T::SchemaModelMaxBytesBoundedVecLimit::get() - 1 },
		>,
	) -> Result<(), BenchmarkError> {
		let sender: T::AccountId = whitelisted_caller();
		let namespace = vec![b'a'; NAMESPACE_MIN as usize];
		let descriptor = vec![b'b'; DESCRIPTOR_MAX as usize];
		let name: Vec<u8> = namespace
			.into_iter()
			.chain(vec![b'.'].into_iter())
			.chain(descriptor.into_iter())
			.collect();
		let bounded_name = BoundedVec::try_from(name).expect("should resolve");
		let model_type = ModelType::AvroBinary;
		let payload_location = PayloadLocation::OnChain;
		assert_ok!(SchemasPallet::<T>::set_max_schema_model_bytes(
			RawOrigin::Root.into(),
			T::SchemaModelMaxBytesBoundedVecLimit::get()
		));
		let schema_input = generate_schema::<T>(m as usize);

		#[extrinsic_call]
		create_schema_v3(
			RawOrigin::Signed(sender),
			schema_input,
			model_type,
			payload_location,
			BoundedVec::default(),
			Some(bounded_name),
		);

		ensure!(
			CurrentSchemaIdentifierMaximum::<T>::get() > 0,
			"Created schema count should be > 0"
		);
		ensure!(SchemaInfos::<T>::get(1).is_some(), "Created schema should exist");
		Ok(())
	}

	#[benchmark]
	fn create_schema_v3_without_name(
		m: Linear<
			{ T::MinSchemaModelSizeBytes::get() + 8 },
			{ T::SchemaModelMaxBytesBoundedVecLimit::get() - 1 },
		>,
	) -> Result<(), BenchmarkError> {
		let sender: T::AccountId = whitelisted_caller();
		let model_type = ModelType::AvroBinary;
		let payload_location = PayloadLocation::OnChain;
		assert_ok!(SchemasPallet::<T>::set_max_schema_model_bytes(
			RawOrigin::Root.into(),
			T::SchemaModelMaxBytesBoundedVecLimit::get()
		));
		let schema_input = generate_schema::<T>(m as usize);

		#[extrinsic_call]
		create_schema_v3(
			RawOrigin::Signed(sender),
			schema_input,
			model_type,
			payload_location,
			BoundedVec::default(),
			None,
		);

		ensure!(
			CurrentSchemaIdentifierMaximum::<T>::get() > 0,
			"Created schema count should be > 0"
		);
		ensure!(SchemaInfos::<T>::get(1).is_some(), "Created schema should exist");
		Ok(())
	}

	#[benchmark]
	fn set_max_schema_model_bytes() -> Result<(), BenchmarkError> {
		let sender = RawOrigin::Root;
		let max_size = T::SchemaModelMaxBytesBoundedVecLimit::get();

		#[extrinsic_call]
		_(sender, max_size);

		ensure!(
			GovernanceSchemaModelMaxBytes::<T>::get() ==
				T::SchemaModelMaxBytesBoundedVecLimit::get(),
			"Schema model max should be updated!"
		);
		Ok(())
	}

	#[benchmark]
	fn create_schema_via_governance_v2_with_name(
		m: Linear<
			{ T::MinSchemaModelSizeBytes::get() + 8 },
			{ T::SchemaModelMaxBytesBoundedVecLimit::get() - 1 },
		>,
	) -> Result<(), BenchmarkError> {
		let sender: T::AccountId = whitelisted_caller();
		let namespace = vec![b'a'; NAMESPACE_MIN as usize];
		let descriptor = vec![b'b'; DESCRIPTOR_MAX as usize];
		let name: Vec<u8> = namespace
			.into_iter()
			.chain(vec![b'.'].into_iter())
			.chain(descriptor.into_iter())
			.collect();
		let bounded_name = BoundedVec::try_from(name).expect("should resolve");
		let model_type = ModelType::AvroBinary;
		let payload_location = PayloadLocation::OnChain;
		assert_ok!(SchemasPallet::<T>::set_max_schema_model_bytes(
			RawOrigin::Root.into(),
			T::SchemaModelMaxBytesBoundedVecLimit::get()
		));
		let schema_input = generate_schema::<T>(m as usize);

		#[extrinsic_call]
		create_schema_via_governance_v2(
			RawOrigin::Root,
			sender.clone(),
			schema_input,
			model_type,
			payload_location,
			BoundedVec::default(),
			Some(bounded_name),
		);

		ensure!(
			CurrentSchemaIdentifierMaximum::<T>::get() > 0,
			"Created schema count should be > 0"
		);
		ensure!(SchemaInfos::<T>::get(1).is_some(), "Created schema should exist");
		Ok(())
	}

	#[benchmark]
	fn create_schema_via_governance_v2_without_name(
		m: Linear<
			{ T::MinSchemaModelSizeBytes::get() + 8 },
			{ T::SchemaModelMaxBytesBoundedVecLimit::get() - 1 },
		>,
	) -> Result<(), BenchmarkError> {
		let sender: T::AccountId = whitelisted_caller();
		let model_type = ModelType::AvroBinary;
		let payload_location = PayloadLocation::OnChain;
		assert_ok!(SchemasPallet::<T>::set_max_schema_model_bytes(
			RawOrigin::Root.into(),
			T::SchemaModelMaxBytesBoundedVecLimit::get()
		));
		let schema_input = generate_schema::<T>(m as usize);

		#[extrinsic_call]
		create_schema_via_governance_v2(
			RawOrigin::Root,
			sender.clone(),
			schema_input,
			model_type,
			payload_location,
			BoundedVec::default(),
			None,
		);

		ensure!(
			CurrentSchemaIdentifierMaximum::<T>::get() > 0,
			"Created schema count should be > 0"
		);
		ensure!(SchemaInfos::<T>::get(1).is_some(), "Created schema should exist");
		Ok(())
	}

	#[benchmark]
	fn propose_to_create_schema_v2_with_name(
		m: Linear<
			{ T::MinSchemaModelSizeBytes::get() + 8 },
			{ T::SchemaModelMaxBytesBoundedVecLimit::get() - 1 },
		>,
	) -> Result<(), BenchmarkError> {
		let sender: T::AccountId = whitelisted_caller();
		let model_type = ModelType::AvroBinary;
		let payload_location = PayloadLocation::OnChain;
		let namespace = vec![b'a'; NAMESPACE_MIN as usize];
		let descriptor = vec![b'b'; DESCRIPTOR_MAX as usize];
		let name: Vec<u8> = namespace
			.into_iter()
			.chain(vec![b'.'].into_iter())
			.chain(descriptor.into_iter())
			.collect();
		let bounded_name = BoundedVec::try_from(name).expect("should resolve");
		assert_ok!(SchemasPallet::<T>::set_max_schema_model_bytes(
			RawOrigin::Root.into(),
			T::SchemaModelMaxBytesBoundedVecLimit::get()
		));
		let schema_input = generate_schema::<T>(m as usize);

		#[extrinsic_call]
		propose_to_create_schema_v2(
			RawOrigin::Signed(sender),
			schema_input,
			model_type,
			payload_location,
			BoundedVec::default(),
			Some(bounded_name),
		);

		assert_eq!(T::ProposalProvider::proposal_count(), 1);
		Ok(())
	}

	#[benchmark]
	fn propose_to_create_schema_v2_without_name(
		m: Linear<
			{ T::MinSchemaModelSizeBytes::get() + 8 },
			{ T::SchemaModelMaxBytesBoundedVecLimit::get() - 1 },
		>,
	) -> Result<(), BenchmarkError> {
		let sender: T::AccountId = whitelisted_caller();
		let model_type = ModelType::AvroBinary;
		let payload_location = PayloadLocation::OnChain;
		assert_ok!(SchemasPallet::<T>::set_max_schema_model_bytes(
			RawOrigin::Root.into(),
			T::SchemaModelMaxBytesBoundedVecLimit::get()
		));
		let schema_input = generate_schema::<T>(m as usize);

		#[extrinsic_call]
		propose_to_create_schema_v2(
			RawOrigin::Signed(sender),
			schema_input,
			model_type,
			payload_location,
			BoundedVec::default(),
			None,
		);

		assert_eq!(T::ProposalProvider::proposal_count(), 1);
		Ok(())
	}

	#[benchmark]
	fn propose_to_create_schema_name() -> Result<(), BenchmarkError> {
		let sender: T::AccountId = whitelisted_caller();
		let model = generate_schema::<T>(100 as usize);
		let namespace = vec![b'a'; NAMESPACE_MIN as usize];
		let descriptor = vec![b'b'; DESCRIPTOR_MAX as usize];
		let name: Vec<u8> = namespace
			.into_iter()
			.chain(vec![b'.'].into_iter())
			.chain(descriptor.into_iter())
			.collect();
		let schema_name = SchemaNamePayload::try_from(name).expect("should resolve");
		assert_ok!(SchemasPallet::<T>::set_max_schema_model_bytes(
			RawOrigin::Root.into(),
			T::SchemaModelMaxBytesBoundedVecLimit::get()
		));
		let schema_id = SchemasPallet::<T>::add_schema(
			model,
			ModelType::AvroBinary,
			PayloadLocation::OnChain,
			BoundedVec::default(),
			None,
		)
		.map_err(|e| BenchmarkError::Stop(e.into()))?;

		#[extrinsic_call]
		_(RawOrigin::Signed(sender), schema_id, schema_name);

		assert_eq!(T::ProposalProvider::proposal_count(), 1);
		Ok(())
	}

	#[benchmark]
	fn create_schema_name_via_governance() -> Result<(), BenchmarkError> {
		let model = generate_schema::<T>(100 as usize);
		let namespace = vec![b'a'; NAMESPACE_MIN as usize];
		let descriptor = vec![b'b'; DESCRIPTOR_MAX as usize];
		let name: Vec<u8> = namespace
			.into_iter()
			.chain(vec![b'.'].into_iter())
			.chain(descriptor.into_iter())
			.collect();
		let schema_name = SchemaNamePayload::try_from(name).expect("should resolve");
		assert_ok!(SchemasPallet::<T>::set_max_schema_model_bytes(
			RawOrigin::Root.into(),
			T::SchemaModelMaxBytesBoundedVecLimit::get()
		));
		let schema_id = SchemasPallet::<T>::add_schema(
			model,
			ModelType::AvroBinary,
			PayloadLocation::OnChain,
			BoundedVec::default(),
			None,
		)
		.map_err(|e| BenchmarkError::Stop(e.into()))?;

		#[extrinsic_call]
		_(RawOrigin::Root, schema_id, schema_name.clone());

		let versions = SchemasPallet::<T>::get_schema_versions(schema_name.into_inner());
		ensure!(versions.is_some(), "Created schema name should exist");
		ensure!(versions.unwrap().len() == 1, "Version should be added!");
		Ok(())
	}

	impl_benchmark_test_suite!(
		SchemasPallet,
		crate::tests::mock::new_test_ext(),
		crate::tests::mock::Test
	);
}
