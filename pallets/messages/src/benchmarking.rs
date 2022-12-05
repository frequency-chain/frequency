#![cfg(feature = "runtime-benchmarks")]

use super::*;
#[allow(unused)]
use crate::Pallet as MessagesPallet;
use common_primitives::{
	msa::{DelegatorId, ProviderId},
	schema::*,
};
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::{assert_ok, pallet_prelude::DispatchResult, traits::OnInitialize};
use frame_system::RawOrigin;

const AVERAGE_NUMBER_OF_MESSAGES: u32 = 499;
const SCHEMAS: u32 = 50;
const IPFS_SCHEMA_ID: u16 = 50;
const IPFS_PAYLOAD_LENGTH: u32 = 10;

fn onchain_message<T: Config>(schema_id: SchemaId) -> DispatchResult {
	let message_source_id = DelegatorId(1);
	let provider_id = ProviderId(1);
	let payload = Vec::from(
		"{'fromId': 123, 'content': '232323114432', 'fromId': 123, 'content': '232323114432'}"
			.as_bytes(),
	);
	let bounded_payload: BoundedVec<u8, T::MaxMessagePayloadSizeBytes> =
		payload.try_into().expect("Invalid payload");
	MessagesPallet::<T>::add_message(
		provider_id.into(),
		Some(message_source_id.into()),
		bounded_payload,
		schema_id,
	)?;
	Ok(())
}

/// Helper function to call MessagesPallet::<T>::add_ipfs_message
fn ipfs_message<T: Config>(schema_id: SchemaId) -> DispatchResult {
	let payload =
		Vec::from("bafkreidgvpkjawlxz6sffxzwgooowe5yt7i6wsyg236mfoks77nywkptdq".as_bytes());
	let provider_id = ProviderId(1);
	let bounded_payload: BoundedVec<u8, T::MaxMessagePayloadSizeBytes> =
		payload.try_into().expect("Invalid payload");

	MessagesPallet::<T>::add_message(provider_id.into(), None, bounded_payload, schema_id)?;

	Ok(())
}

fn create_schema<T: Config>(location: PayloadLocation) -> DispatchResult {
	T::SchemaBenchmarkHelper::create_schema(
		Vec::from(r#"{"Name": "Bond", "Code": "007"}"#.as_bytes()),
		ModelType::AvroBinary,
		location,
	)
}

benchmarks! {
	add_onchain_message {
		let n in 0 .. T::MaxMessagePayloadSizeBytes::get() - 1;
		let message_source_id = DelegatorId(2);
		let caller: T::AccountId = whitelisted_caller();
		let schema_id = 1;

		// schema ids start from 1, and we need to add that many to make sure our desired id exists
		for j in 0 ..=schema_id {
			assert_ok!(create_schema::<T>(PayloadLocation::OnChain));
		}
		assert_ok!(T::MsaBenchmarkHelper::add_key(ProviderId(1).into(), caller.clone()));
		assert_ok!(T::MsaBenchmarkHelper::set_delegation_relationship(ProviderId(1), message_source_id.into(), [schema_id].to_vec()));

		let payload = vec![1; n as usize];

		for j in 1 .. AVERAGE_NUMBER_OF_MESSAGES {
			assert_ok!(onchain_message::<T>(schema_id));
		}
	}: _ (RawOrigin::Signed(caller), Some(message_source_id.into()), schema_id, payload)

	add_ipfs_message {
		let n in 0 .. T::MaxMessagePayloadSizeBytes::get() - IPFS_PAYLOAD_LENGTH;
		let caller: T::AccountId = whitelisted_caller();
		let cid = vec![1; n as usize];

		// schema ids start from 1, and we need to add that many to make sure our desired id exists
		for j in 0 ..=IPFS_SCHEMA_ID {
			assert_ok!(create_schema::<T>(PayloadLocation::IPFS));
		}
		assert_ok!(T::MsaBenchmarkHelper::add_key(ProviderId(1).into(), caller.clone()));

		for j in 1 .. AVERAGE_NUMBER_OF_MESSAGES {
			assert_ok!(ipfs_message::<T>(IPFS_SCHEMA_ID));
		}
	}: _ (RawOrigin::Signed(caller),IPFS_SCHEMA_ID, cid, IPFS_PAYLOAD_LENGTH)

	on_initialize {
		let m in 1 .. AVERAGE_NUMBER_OF_MESSAGES;
		let s in 1 .. SCHEMAS;

		for j in 0 .. m {
			let schema_id = j % s;
			assert_ok!(onchain_message::<T>(schema_id.try_into().unwrap()));
		}

	}: {
		MessagesPallet::<T>::on_initialize(2u32.into());
	}
	verify {
		assert_eq!(BlockMessages::<T>::get().len(), 0);
	}

	impl_benchmark_test_suite!(MessagesPallet, crate::mock::new_test_ext(), crate::mock::Test);
}
