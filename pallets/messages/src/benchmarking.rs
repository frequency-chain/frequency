use super::*;
#[allow(unused)]
use crate::Pallet as MessagesPallet;
use common_primitives::schema::*;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::{assert_ok, pallet_prelude::DispatchResultWithPostInfo, traits::OnInitialize};
use frame_system::RawOrigin;

const MESSAGES: u32 = 499;
const SCHEMAS: u32 = 50;

fn add_message<T: Config>(schema_id: SchemaId) -> DispatchResultWithPostInfo {
	let acc: T::AccountId = whitelisted_caller();
	MessagesPallet::<T>::add_onchain_message(
		RawOrigin::Signed(acc.clone()).into(),
		None,
		schema_id,
		Vec::from(
			"{'fromId': 123, 'content': '232323114432', 'fromId': 123, 'content': '232323114432'}"
				.as_bytes(),
		),
	)
}

benchmarks! {
	add_onchain_message {
		let n in 0 .. T::MaxMessagePayloadSizeBytes::get() - 1;
		let m in 1 .. MESSAGES;
		let caller: T::AccountId = whitelisted_caller();
		let input = vec![1; n as usize];

		for j in 0 .. m {
			let sid = j % SCHEMAS;
			assert_ok!(add_message::<T>(sid.try_into().unwrap()));
		}
	}: _ (RawOrigin::Signed(caller), None, 1, input)

	add_ipfs_message {
		let n in 0 .. T::MaxMessagePayloadSizeBytes::get() - 1;
		let m in 1 .. MESSAGES;
		let caller: T::AccountId = whitelisted_caller();
		let input = vec![1; n as usize];
		let cid = CID::new(input);
		let payload_length = 1_000;
		let payload: IPFSPayload = IPFSPayload::new(cid, payload_length);

		for j in 0 .. m {
			let sid = j % SCHEMAS;
			assert_ok!(add_message::<T>(sid.try_into().unwrap()));
		}
	}: _ (RawOrigin::Signed(caller), None, 1, payload)

	on_initialize {
		let m in 1 .. MESSAGES;
		let s in 1 .. SCHEMAS;

		for j in 0 .. m {
			let sid = j % s;
			assert_ok!(add_message::<T>(sid.try_into().unwrap()));
		}

	}: {
		MessagesPallet::<T>::on_initialize(2u32.into());
	}
	verify {
		assert_eq!(BlockMessages::<T>::get().len(), 0);
	}

	impl_benchmark_test_suite!(MessagesPallet, crate::mock::new_test_ext(), crate::mock::Test);
}
