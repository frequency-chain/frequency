#![allow(clippy::expect_used)]

use super::*;
use crate::Pallet as MessagesPallet;
use common_primitives::{
	msa::{DelegatorId, ProviderId},
	schema::*,
};
use frame_benchmarking::v2::*;
use frame_support::{
	assert_ok, migrations::SteppedMigration, pallet_prelude::DispatchResult, weights::WeightMeter,
};
use frame_system::{pallet_prelude::BlockNumberFor, RawOrigin};
use sp_runtime::traits::One;
extern crate alloc;
use alloc::vec;

const IPFS_PAYLOAD_LENGTH: u32 = 10;
const MAX_MESSAGES_IN_BLOCK: u32 = 500;
// Any IPFS schema ID from the [pallet genesis file](../../../resources/genesis-schemas.json) will work
const IPFS_SCHEMA_ID: u16 = 20;

fn onchain_message<T: Config>(intent_id: IntentId, schema_id: SchemaId) -> DispatchResult {
	let message_source_id = DelegatorId(1);
	let provider_id = ProviderId(1);
	let payload = Vec::from(
		"{'fromId': 123, 'content': '232323', 'fromId': 123, 'content': '232323'}".as_bytes(),
	);
	let bounded_payload: BoundedVec<u8, T::MessagesMaxPayloadSizeBytes> =
		payload.try_into().expect("Invalid payload");
	MessagesPallet::<T>::add_message(
		provider_id.into(),
		Some(message_source_id.into()),
		bounded_payload,
		intent_id,
		schema_id,
		BlockNumberFor::<T>::one(),
	)?;
	Ok(())
}

/// Helper function to call MessagesPallet::<T>::add_ipfs_message
fn ipfs_message<T: Config>(intent_id: IntentId, schema_id: SchemaId) -> DispatchResult {
	let payload =
		Vec::from("bafkreidgvpkjawlxz6sffxzwgooowe5yt7i6wsyg236mfoks77nywkptdq".as_bytes());
	let provider_id = ProviderId(1);
	let bounded_payload: BoundedVec<u8, T::MessagesMaxPayloadSizeBytes> =
		payload.try_into().expect("Invalid payload");

	MessagesPallet::<T>::add_message(
		provider_id.into(),
		None,
		bounded_payload,
		intent_id,
		schema_id,
		BlockNumberFor::<T>::one(),
	)?;

	Ok(())
}

#[benchmarks]
mod benchmarks {
	use super::*;
	use frame_support::{pallet_prelude::StorageVersion, traits::GetStorageVersion};

	#[benchmark]
	fn add_onchain_message(
		n: Linear<0, { T::MessagesMaxPayloadSizeBytes::get() - 1 }>,
	) -> Result<(), BenchmarkError> {
		let message_source_id = DelegatorId(2);
		let caller: T::AccountId = whitelisted_caller();

		// The schemas pallet genesis does not contain an OnChain intent/schema, so we create one here
		let intent_id = T::SchemaBenchmarkHelper::create_intent(
			b"benchmark.onchain-intent".to_vec(),
			PayloadLocation::OnChain,
			Vec::default(),
		)?;

		let schema_id = T::SchemaBenchmarkHelper::create_schema(
			intent_id,
			Vec::from(r#"{"Name": "Bond", "Code": "007"}"#.as_bytes()),
			ModelType::AvroBinary,
			PayloadLocation::OnChain,
		)?;

		assert_ok!(T::MsaBenchmarkHelper::add_key(ProviderId(1).into(), caller.clone()));
		assert_ok!(T::MsaBenchmarkHelper::set_delegation_relationship(
			ProviderId(1),
			message_source_id,
			[schema_id].to_vec()
		));

		let payload = vec![1; n as usize];
		for _ in 1..MAX_MESSAGES_IN_BLOCK {
			assert_ok!(onchain_message::<T>(intent_id, schema_id));
		}

		#[extrinsic_call]
		_(RawOrigin::Signed(caller), Some(message_source_id.into()), schema_id, payload);

		assert_eq!(
			MessagesPallet::<T>::get_messages_by_intent_and_block(
				intent_id,
				PayloadLocation::OnChain,
				BlockNumberFor::<T>::one()
			)
			.len(),
			MAX_MESSAGES_IN_BLOCK as usize
		);

		Ok(())
	}

	#[benchmark]
	fn add_ipfs_message() -> Result<(), BenchmarkError> {
		let caller: T::AccountId = whitelisted_caller();
		let cid = "bafkreidgvpkjawlxz6sffxzwgooowe5yt7i6wsyg236mfoks77nywkptdq"
			.as_bytes()
			.to_vec();
		let schema_id = IPFS_SCHEMA_ID;
		let intent_id = IPFS_SCHEMA_ID as IntentId;

		assert_ok!(T::MsaBenchmarkHelper::add_key(ProviderId(1).into(), caller.clone()));
		for _ in 1..MAX_MESSAGES_IN_BLOCK {
			assert_ok!(ipfs_message::<T>(intent_id, schema_id));
		}

		#[extrinsic_call]
		_(RawOrigin::Signed(caller), schema_id, cid, IPFS_PAYLOAD_LENGTH);

		assert_eq!(
			MessagesPallet::<T>::get_messages_by_intent_and_block(
				intent_id,
				PayloadLocation::IPFS,
				BlockNumberFor::<T>::one()
			)
			.len(),
			MAX_MESSAGES_IN_BLOCK as usize
		);
		Ok(())
	}

	/// Benchmark a single step of the `v3::MigrateV2ToV3` migration. Here we benchmark the cost
	/// to migrate a _single record_. This weight is then used in the migration itself for self-metering.
	#[benchmark]
	fn v2_to_v3_step() {
		let payload: BoundedVec<u8, T::MessagesMaxPayloadSizeBytes> =
			vec![1; 3072].try_into().expect("Unable to create BoundedVec payload");
		let old_message = migration::v2::Message {
			payload: payload.clone(),
			provider_msa_id: 1u64,
			msa_id: Some(1u64),
		};
		migration::v2::MessagesV2::<T>::insert(
			(BlockNumberFor::<T>::default(), 0u16, 0u16),
			old_message,
		);
		let mut meter = WeightMeter::new();

		#[block]
		{
			migration::v3::MigrateV2ToV3::<T, weights::SubstrateWeight<T>>::step(None, &mut meter)
				.expect("migration call failed");
		}

		// Check that the new storage is decodable:
		let new_message =
			crate::MessagesV3::<T>::try_get((BlockNumberFor::<T>::default(), 0u16, 0u16));
		assert!(new_message.is_ok());
		let new_message = new_message.expect("Unable to fetch migrated message");
		assert_eq!(Some(1u64), new_message.msa_id);
		assert_eq!(1u64, new_message.provider_msa_id);
		assert_eq!(payload, new_message.payload);
		assert_eq!(new_message.schema_id, 0u16);
	}

	/// Benchmark a single step of the `v3::FinalizeV3Migration` migration. Here we benchmark the cost
	/// to migrate a _single record_. This weight is then used in the migration itself for self-metering.
	#[benchmark]
	fn v2_to_v3_final_step() {
		let mut meter = WeightMeter::new();

		#[block]
		{
			migration::v3::FinalizeV3Migration::<T, weights::SubstrateWeight<T>>::step(
				None, &mut meter,
			)
			.expect("final storage version migration failed");
		}

		// Check that the storage version was correctly set
		let new_version = Pallet::<T>::on_chain_storage_version();
		assert_eq!(new_version, StorageVersion::new(3));
	}

	impl_benchmark_test_suite!(
		MessagesPallet,
		crate::tests::mock::new_test_ext(),
		crate::tests::mock::Test
	);
}
