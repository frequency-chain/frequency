use super::*;

#[allow(unused)]
use crate::Pallet as MessagesPallet;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::{assert_ok, traits::OnInitialize};
use frame_system::RawOrigin;

const MESSAGES: u32 = 499;
const SCHEMAS: u32 = 50;

fn add_message<T: Config>(schema_id: SchemaId) -> DispatchResult {
	let acc: T::AccountId = whitelisted_caller();
	MessagesPallet::<T>::add(
		RawOrigin::Signed(acc.clone()).into(),
		schema_id,
		Vec::from(
			"{'fromId': 123, 'content': '232323114432', 'fromId': 123, 'content': '232323114432'}"
				.as_bytes(),
		),
	)
}

benchmarks! {
	add {
		let n in 0 .. T::MaxMessageSizeInBytes::get() - 1;
		let m in 1 .. MESSAGES;
		let caller: T::AccountId = whitelisted_caller();
		let input = vec![1; n as usize];

		for j in 0 .. m {
			let sid = j % SCHEMAS;
			assert_ok!(add_message::<T>(sid.try_into().unwrap()));
		}
	}: _ (RawOrigin::Signed(caller), 1, input)

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
