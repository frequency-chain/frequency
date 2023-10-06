#![cfg(feature = "runtime-benchmarks")]
#![allow(clippy::expect_used)]

use super::*;
#[allow(unused)]
use crate::Pallet as MessagesPallet;
use common_primitives::{
	msa::{DelegatorId, ProviderId},
	schema::*,
};
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::{assert_ok, pallet_prelude::DispatchResult};
use frame_system::RawOrigin;
use sp_runtime::traits::One;

const AVERAGE_NUMBER_OF_MESSAGES: u32 = 499;
const IPFS_SCHEMA_ID: u16 = 50;
const IPFS_PAYLOAD_LENGTH: u32 = 10;

fn onchain_message<T: Config>(schema_id: SchemaId) -> DispatchResult {
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
		schema_id,
		<T as frame_system::Config>::BlockNumber::one(),
	)?;
	Ok(())
}

/// Helper function to call MessagesPallet::<T>::add_ipfs_message
fn ipfs_message<T: Config>(schema_id: SchemaId) -> DispatchResult {
	let payload =
		Vec::from("bafkreidgvpkjawlxz6sffxzwgooowe5yt7i6wsyg236mfoks77nywkptdq".as_bytes());
	let provider_id = ProviderId(1);
	let bounded_payload: BoundedVec<u8, T::MessagesMaxPayloadSizeBytes> =
		payload.try_into().expect("Invalid payload");

	MessagesPallet::<T>::add_message(
		provider_id.into(),
		None,
		bounded_payload,
		schema_id,
		<T as frame_system::Config>::BlockNumber::one(),
	)?;

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
		let n in 0 .. T::MessagesMaxPayloadSizeBytes::get() - 1;
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
	verify {
		assert_eq!(
			MessagesPallet::<T>::get_messages(
				<T as frame_system::Config>::BlockNumber::one(), schema_id).len(),
			AVERAGE_NUMBER_OF_MESSAGES as usize
		);
	}

	add_ipfs_message {
		let caller: T::AccountId = whitelisted_caller();
		let cid = "bafkreidgvpkjawlxz6sffxzwgooowe5yt7i6wsyg236mfoks77nywkptdq".as_bytes().to_vec();

		// schema ids start from 1, and we need to add that many to make sure our desired id exists
		for j in 0 ..=IPFS_SCHEMA_ID {
			assert_ok!(create_schema::<T>(PayloadLocation::IPFS));
		}
		assert_ok!(T::MsaBenchmarkHelper::add_key(ProviderId(1).into(), caller.clone()));

		for j in 1 .. AVERAGE_NUMBER_OF_MESSAGES {
			assert_ok!(ipfs_message::<T>(IPFS_SCHEMA_ID));
		}
	}: _ (RawOrigin::Signed(caller),IPFS_SCHEMA_ID, cid, IPFS_PAYLOAD_LENGTH)
	verify {
		assert_eq!(
			MessagesPallet::<T>::get_messages(
				<T as frame_system::Config>::BlockNumber::one(), IPFS_SCHEMA_ID).len(),
			AVERAGE_NUMBER_OF_MESSAGES as usize
		);
	}

	impl_benchmark_test_suite!(MessagesPallet, crate::tests::mock::new_test_ext(), crate::tests::mock::Test);
}
