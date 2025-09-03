#![allow(clippy::unwrap_used)]
extern crate alloc;
use super::*;
use crate::Pallet as SchemasPallet;
use alloc::vec;
use common_primitives::schema::IntentSetting;
use frame_benchmarking::{v2::*, whitelisted_caller};
use frame_support::{assert_ok, ensure, BoundedVec};
use frame_system::RawOrigin;
use numtoa::NumToA;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

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
			break;
		}
	}
	json.pop(); // removing last ,
	json.extend(b"}");
	json.try_into().unwrap()
}

fn generate_settings<T: Config>(
	size: usize,
) -> BoundedVec<IntentSetting, T::MaxSchemaSettingsPerSchema> {
	let mut settings: Vec<IntentSetting> = vec![];
	for _ in 0..size {
		settings.push(IntentSetting::SignatureRequired)
	}
	settings.try_into().unwrap()
}

fn generate_intents<T: Config>(size: usize) -> BoundedVec<IntentId, T::MaxIntentsPerIntentGroup> {
	let mut intents: Vec<IntentId> = vec![];
	for id in 0..size {
		let namespace = vec![b'a'; PROTOCOL_NAME_MIN as usize];
		let descriptor = vec![b'b'; DESCRIPTOR_MIN as usize];
		let mut buff = [0u8; 30];
		let name: Vec<u8> = namespace
			.into_iter()
			.chain(vec![b'.'; 1])
			.chain(descriptor.into_iter())
			.chain(id.numtoa(10, &mut buff).iter().copied())
			.collect();
		let name_payload = BoundedVec::try_from(name).expect("should convert");
		let (intent_id, _) = SchemasPallet::<T>::create_intent_for(
			name_payload,
			PayloadLocation::Paginated,
			BoundedVec::default(),
		)
		.expect("should create intent");
		intents.push(intent_id);
	}
	intents.try_into().unwrap()
}

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn create_schema_v4(
		m: Linear<
			{ T::MinSchemaModelSizeBytes::get() + 8 },
			{ T::SchemaModelMaxBytesBoundedVecLimit::get() - 1 },
		>,
	) -> Result<(), BenchmarkError> {
		let sender: T::AccountId = whitelisted_caller();
		let model_type = ModelType::AvroBinary;
		assert_ok!(SchemasPallet::<T>::set_max_schema_model_bytes(
			RawOrigin::Root.into(),
			T::SchemaModelMaxBytesBoundedVecLimit::get()
		));
		let schema_input = generate_schema::<T>(m as usize);
		let intent_id = 1u16;

		#[extrinsic_call]
		create_schema_v4(RawOrigin::Signed(sender), intent_id, schema_input, model_type);

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
	fn create_schema_via_governance_v3(
		m: Linear<
			{ T::MinSchemaModelSizeBytes::get() + 8 },
			{ T::SchemaModelMaxBytesBoundedVecLimit::get() - 1 },
		>,
	) -> Result<(), BenchmarkError> {
		let sender: T::AccountId = whitelisted_caller();
		let model_type = ModelType::AvroBinary;
		assert_ok!(SchemasPallet::<T>::set_max_schema_model_bytes(
			RawOrigin::Root.into(),
			T::SchemaModelMaxBytesBoundedVecLimit::get()
		));
		let schema_input = generate_schema::<T>(m as usize);
		let intent_id = 1u16;

		#[extrinsic_call]
		create_schema_via_governance_v3(
			RawOrigin::Root,
			sender.clone(),
			intent_id,
			schema_input,
			model_type,
		);

		ensure!(
			CurrentSchemaIdentifierMaximum::<T>::get() > 0,
			"Created schema count should be > 0"
		);
		ensure!(SchemaInfos::<T>::get(1).is_some(), "Created schema should exist");
		Ok(())
	}

	#[benchmark]
	fn propose_to_create_schema_v3(
		m: Linear<
			{ T::MinSchemaModelSizeBytes::get() + 8 },
			{ T::SchemaModelMaxBytesBoundedVecLimit::get() - 1 },
		>,
	) -> Result<(), BenchmarkError> {
		let sender: T::AccountId = whitelisted_caller();
		let model_type = ModelType::AvroBinary;
		assert_ok!(SchemasPallet::<T>::set_max_schema_model_bytes(
			RawOrigin::Root.into(),
			T::SchemaModelMaxBytesBoundedVecLimit::get()
		));
		let schema_input = generate_schema::<T>(m as usize);
		let intent_id = 1u16;

		#[extrinsic_call]
		propose_to_create_schema_v3(RawOrigin::Signed(sender), intent_id, schema_input, model_type);

		assert_eq!(T::ProposalProvider::proposal_count(), 1);
		Ok(())
	}

	#[benchmark]
	fn create_intent(
		m: Linear<0, { T::MaxSchemaSettingsPerSchema::get() }>,
	) -> Result<(), BenchmarkError> {
		let sender: T::AccountId = whitelisted_caller();
		let namespace = vec![b'a'; PROTOCOL_NAME_MIN as usize];
		let descriptor = vec![b'b'; DESCRIPTOR_MAX as usize];
		let name: Vec<u8> = namespace
			.into_iter()
			.chain(vec![b'.'].into_iter())
			.chain(descriptor.into_iter())
			.collect();
		let bounded_name = BoundedVec::try_from(name).expect("should resolve");
		let payload_location = PayloadLocation::Itemized;
		let settings = generate_settings::<T>(m as usize);

		#[extrinsic_call]
		create_intent(RawOrigin::Signed(sender), bounded_name, payload_location, settings);

		ensure!(
			CurrentIntentIdentifierMaximum::<T>::get() > 0,
			"Created intent count should be > 0"
		);
		ensure!(IntentInfos::<T>::get(1).is_some(), "Created intent should exist");
		Ok(())
	}

	#[benchmark]
	fn create_intent_via_governance(
		m: Linear<0, { T::MaxSchemaSettingsPerSchema::get() }>,
	) -> Result<(), BenchmarkError> {
		let sender: T::AccountId = whitelisted_caller();
		let namespace = vec![b'a'; PROTOCOL_NAME_MIN as usize];
		let descriptor = vec![b'b'; DESCRIPTOR_MAX as usize];
		let name: Vec<u8> = namespace
			.into_iter()
			.chain(vec![b'.'].into_iter())
			.chain(descriptor.into_iter())
			.collect();
		let bounded_name = BoundedVec::try_from(name).expect("should resolve");
		let payload_location = PayloadLocation::Itemized;
		let settings = generate_settings::<T>(m as usize);

		#[extrinsic_call]
		create_intent_via_governance(
			RawOrigin::Root,
			sender.clone(),
			payload_location,
			settings,
			bounded_name,
		);

		ensure!(
			CurrentIntentIdentifierMaximum::<T>::get() > 0,
			"Created intent count should be > 0"
		);
		ensure!(IntentInfos::<T>::get(1).is_some(), "Created intent should exist");
		Ok(())
	}

	#[benchmark]
	fn propose_to_create_intent() -> Result<(), BenchmarkError> {
		let sender: T::AccountId = whitelisted_caller();
		let payload_location = PayloadLocation::OnChain;
		let namespace = vec![b'a'; PROTOCOL_NAME_MIN as usize];
		let descriptor = vec![b'b'; DESCRIPTOR_MAX as usize];
		let name: Vec<u8> = namespace
			.into_iter()
			.chain(vec![b'.'].into_iter())
			.chain(descriptor.into_iter())
			.collect();
		let bounded_name = BoundedVec::try_from(name).expect("should resolve");

		#[extrinsic_call]
		propose_to_create_intent(
			RawOrigin::Signed(sender),
			payload_location,
			BoundedVec::default(),
			bounded_name,
		);

		assert_eq!(T::ProposalProvider::proposal_count(), 1);
		Ok(())
	}

	#[benchmark]
	fn create_intent_group(
		m: Linear<0, { T::MaxIntentsPerIntentGroup::get() }>,
	) -> Result<(), BenchmarkError> {
		let sender: T::AccountId = whitelisted_caller();
		let namespace = vec![b'a'; PROTOCOL_NAME_MIN as usize];
		let descriptor = vec![b'b'; DESCRIPTOR_MAX as usize];
		let name: Vec<u8> = namespace
			.into_iter()
			.chain(vec![b'.'].into_iter())
			.chain(descriptor.into_iter())
			.collect();
		let bounded_name = BoundedVec::try_from(name).expect("should resolve");
		let intent_ids = generate_intents::<T>(m.try_into().expect("should convert"));

		#[extrinsic_call]
		create_intent_group(RawOrigin::Signed(sender), bounded_name, intent_ids);

		ensure!(
			CurrentIntentGroupIdentifierMaximum::<T>::get() > 0,
			"Created intent group count should be > 0"
		);
		ensure!(IntentGroups::<T>::get(1).is_some(), "Created intent group should exist");
		Ok(())
	}

	#[benchmark]
	fn create_intent_group_via_governance(
		m: Linear<0, { T::MaxSchemaSettingsPerSchema::get() }>,
	) -> Result<(), BenchmarkError> {
		let sender: T::AccountId = whitelisted_caller();
		let namespace = vec![b'a'; PROTOCOL_NAME_MIN as usize];
		let descriptor = vec![b'b'; DESCRIPTOR_MAX as usize];
		let name: Vec<u8> = namespace
			.into_iter()
			.chain(vec![b'.'].into_iter())
			.chain(descriptor.into_iter())
			.collect();
		let bounded_name = BoundedVec::try_from(name).expect("should resolve");
		let intent_ids = generate_intents::<T>(m.try_into().expect("should convert"));

		#[extrinsic_call]
		create_intent_group_via_governance(
			RawOrigin::Root,
			sender.clone(),
			bounded_name,
			intent_ids,
		);

		ensure!(
			CurrentIntentGroupIdentifierMaximum::<T>::get() > 0,
			"Created intent group count should be > 0"
		);
		ensure!(IntentGroups::<T>::get(1).is_some(), "Created intent group should exist");
		Ok(())
	}

	#[benchmark]
	fn propose_to_create_intent_group() -> Result<(), BenchmarkError> {
		let sender: T::AccountId = whitelisted_caller();
		let namespace = vec![b'a'; PROTOCOL_NAME_MIN as usize];
		let descriptor = vec![b'b'; DESCRIPTOR_MAX as usize];
		let name: Vec<u8> = namespace
			.into_iter()
			.chain(vec![b'.'].into_iter())
			.chain(descriptor.into_iter())
			.collect();
		let bounded_name = BoundedVec::try_from(name).expect("should convert");
		let intent_ids = generate_intents::<T>(0);

		#[extrinsic_call]
		propose_to_create_intent_group(RawOrigin::Signed(sender), bounded_name, intent_ids);

		assert_eq!(T::ProposalProvider::proposal_count(), 1);
		Ok(())
	}

	#[benchmark]
	fn update_intent_group(
		m: Linear<0, { T::MaxIntentsPerIntentGroup::get() }>,
	) -> Result<(), BenchmarkError> {
		let sender: T::AccountId = whitelisted_caller();
		let namespace = vec![b'a'; PROTOCOL_NAME_MIN as usize];
		let descriptor = vec![b'b'; DESCRIPTOR_MAX as usize];
		let name: Vec<u8> = namespace
			.into_iter()
			.chain(vec![b'.'].into_iter())
			.chain(descriptor.into_iter())
			.collect();
		let bounded_name = BoundedVec::try_from(name).expect("should resolve");
		let intent_ids = generate_intents::<T>(T::MaxIntentsPerIntentGroup::get() as usize);
		let mut initial_intent_ids = intent_ids.clone();
		initial_intent_ids.truncate(1);

		let (intent_group_id, _) =
			SchemasPallet::<T>::create_intent_group_for(bounded_name, initial_intent_ids)
				.expect("should create intent group");

		#[extrinsic_call]
		update_intent_group(
			RawOrigin::Signed(sender.clone()),
			intent_group_id,
			BoundedVec::try_from(intent_ids.as_slice()[0..m as usize].to_vec()).unwrap(),
		);

		assert_last_event::<T>(
			Event::<T>::IntentGroupUpdated { key: sender, intent_group_id }.into(),
		);
		Ok(())
	}

	#[benchmark]
	fn update_intent_group_via_governance(
		m: Linear<0, { T::MaxSchemaSettingsPerSchema::get() }>,
	) -> Result<(), BenchmarkError> {
		let sender: T::AccountId = whitelisted_caller();
		let namespace = vec![b'a'; PROTOCOL_NAME_MIN as usize];
		let descriptor = vec![b'b'; DESCRIPTOR_MAX as usize];
		let name: Vec<u8> = namespace
			.into_iter()
			.chain(vec![b'.'].into_iter())
			.chain(descriptor.into_iter())
			.collect();
		let bounded_name = BoundedVec::try_from(name).expect("should resolve");
		let intent_ids = generate_intents::<T>(T::MaxIntentsPerIntentGroup::get() as usize);
		let mut initial_intent_ids = intent_ids.clone();
		initial_intent_ids.truncate(1);

		let (intent_group_id, _) =
			SchemasPallet::<T>::create_intent_group_for(bounded_name, initial_intent_ids)
				.expect("should create intent group");

		#[extrinsic_call]
		update_intent_group(
			RawOrigin::Signed(sender.clone()),
			intent_group_id,
			BoundedVec::try_from(intent_ids.as_slice()[0..m as usize].to_vec()).unwrap(),
		);

		assert_last_event::<T>(
			Event::<T>::IntentGroupUpdated { key: sender, intent_group_id }.into(),
		);
		Ok(())
	}

	#[benchmark]
	fn propose_to_update_intent_group() -> Result<(), BenchmarkError> {
		let sender: T::AccountId = whitelisted_caller();
		let namespace = vec![b'a'; PROTOCOL_NAME_MIN as usize];
		let descriptor = vec![b'b'; DESCRIPTOR_MAX as usize];
		let name: Vec<u8> = namespace
			.into_iter()
			.chain(vec![b'.'].into_iter())
			.chain(descriptor.into_iter())
			.collect();
		let bounded_name = BoundedVec::try_from(name).expect("should convert");
		let intent_ids = generate_intents::<T>(0);

		#[extrinsic_call]
		propose_to_create_intent_group(RawOrigin::Signed(sender), bounded_name, intent_ids);

		assert_eq!(T::ProposalProvider::proposal_count(), 1);
		Ok(())
	}

	impl_benchmark_test_suite!(
		SchemasPallet,
		crate::tests::mock::new_test_ext(),
		crate::tests::mock::Test
	);
}
