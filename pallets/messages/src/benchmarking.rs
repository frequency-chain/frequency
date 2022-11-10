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

const MESSAGES: u32 = 499;
const SCHEMAS: u32 = 50;
const IPFS_SCHEMA_ID: u16 = 65535;
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

benchmarks! {
	add_onchain_message {
		let n in 0 .. T::MaxMessagePayloadSizeBytes::get() - 1;
		let m in 1 .. MESSAGES;
		let message_source_id = DelegatorId(2);
		let caller: T::AccountId = whitelisted_caller();

		T::Helper::set_schema_count(51);
		assert_ok!(T::Helper::add_key(ProviderId(1).into(), caller.clone()));
		assert_ok!(T::Helper::set_delegation_relationship(ProviderId(1), message_source_id.into(), [1].to_vec()));

		let payload = vec![1; n as usize];

		for j in 0 .. m {
			let schema_id = j % SCHEMAS;
			assert_ok!(onchain_message::<T>(schema_id.try_into().unwrap()));
		}
	}: _ (RawOrigin::Signed(caller), Some(message_source_id.into()), 1, payload)

	add_ipfs_message {
		let n in 0 .. T::MaxMessagePayloadSizeBytes::get() - IPFS_PAYLOAD_LENGTH;
		let m in 1 .. MESSAGES;
		let caller: T::AccountId = whitelisted_caller();
		let cid = vec![1; n as usize];

		assert_ok!(T::Helper::add_key(ProviderId(1).into(), caller.clone()));

		for j in 0 .. m {
			let schema_id = IPFS_SCHEMA_ID;
			assert_ok!(ipfs_message::<T>(schema_id.try_into().unwrap()));
		}
	}: _ (RawOrigin::Signed(caller),IPFS_SCHEMA_ID, cid, IPFS_PAYLOAD_LENGTH)

	on_initialize {
		let m in 1 .. MESSAGES;
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
