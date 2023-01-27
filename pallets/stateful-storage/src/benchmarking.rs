#![cfg(feature = "runtime-benchmarks")]

use super::*;
#[allow(unused)]
use crate::Pallet as StatefulStoragePallet;
use common_primitives::{benchmarks::*, msa::ProviderId, schema::*};
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::RawOrigin;

fn create_schema<T: Config>() -> DispatchResult {
	T::SchemaBenchmarkHelper::create_schema(
		Vec::from(r#"{"Name": "Bond", "Code": "007"}"#.as_bytes()),
		ModelType::AvroBinary,
		PayloadLocation::OnChain,
	)
}

benchmarks! {
	add_item {
		let n in 0 .. 5;
		let caller: T::AccountId = whitelisted_caller();

		let payload = vec![1u8; n as usize];
	}: _ (RawOrigin::Signed(caller), payload)

	remove_item {
		let caller: T::AccountId = whitelisted_caller();
	}: _ (RawOrigin::Signed(caller))

	upsert_page {
		let s in 0 .. T::MaxPaginatedPageSizeBytes::get() - 1;
		let n in 0 .. (T::MaxPaginatedPageCount::get() - 1).into();
		let caller: T::AccountId = whitelisted_caller();
		let schema_id = 5;

		// schema ids start from 1, and we need to add that many to make sure our desired id exists
		for j in 0 ..schema_id {
			assert_ok!(create_schema::<T>());
		}
		assert_ok!(T::MsaBenchmarkHelper::add_key(ProviderId(1).into(), caller.clone()));

		let payload = vec![1u8; s as usize];
		let page_id = n;
	}: _ (RawOrigin::Signed(caller), schema_id, page_id as u16, payload)

	remove_page {
		let caller: T::AccountId = whitelisted_caller();
	}: _ (RawOrigin::Signed(caller))

	impl_benchmark_test_suite!(StatefulStoragePallet,
		crate::mock::new_test_ext(),
		crate::mock::Test);
}
