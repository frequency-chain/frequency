use super::*;
use crate::{types::ItemAction, Pallet as StatefulStoragePallet};
use codec::Encode;
use common_primitives::{
	schema::{ModelType, PayloadLocation, SchemaId},
	stateful_storage::PageId,
};
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::RawOrigin;
use sp_core::bounded::BoundedVec;
use stateful_child_tree::StatefulChildTree;

pub const ITEMIZED_SCHEMA: SchemaId = 100; // keep in sync with mock.rs. TODO: refactor
pub const PAGINATED_SCHEMA: SchemaId = 101; // keep in sync with mock.rs. TODO: refactor

fn itemized_actions_add<T: Config>(
	n: u32,
	s: usize,
) -> BoundedVec<ItemAction, T::MaxItemizedActionsCount> {
	let mut actions = vec![];
	for _ in 0..n {
		let payload = vec![0u8; s];
		actions.push(ItemAction::Add { data: payload.into() });
	}
	actions.try_into().expect("Invalid actions")
}

fn create_schema<T: Config>(location: PayloadLocation) -> DispatchResult {
	T::SchemaBenchmarkHelper::create_schema(
		Vec::from(r#"{"Message": "some-random-hash"}"#.as_bytes()),
		ModelType::AvroBinary,
		location,
	)
}

benchmarks! {
	apply_item_actions {
		let n in 1 .. T::MaxItemizedActionsCount::get() - 1;
		let s in 1 .. T::MaxItemizedBlobSizeBytes::get()- 1;
		let provider_msa_id = 1u64;
		let delegator_msa_id = 2u64;
		let schema_id = ITEMIZED_SCHEMA;
		let caller: T::AccountId = whitelisted_caller();
		let payload = vec![0u8; s as usize];

		assert_ok!(T::MsaBenchmarkHelper::add_key(provider_msa_id.into(), caller.clone()));
		assert_ok!(T::MsaBenchmarkHelper::set_delegation_relationship(provider_msa_id.into(), delegator_msa_id.into(), [ITEMIZED_SCHEMA].to_vec()));

		let actions = itemized_actions_add::<T>(n, s as usize);
	}: _ (RawOrigin::Signed(caller), delegator_msa_id.into(), schema_id, actions)
	verify {
		let page_result = StatefulStoragePallet::<T>::get_itemized_page(delegator_msa_id, schema_id);
		assert!(page_result.is_some());
		assert!(page_result.unwrap().data.len() > 0);
	}

	upsert_page {
		let s in 1 .. T::MaxPaginatedPageSizeBytes::get();
		let provider_msa_id = 1u64;
		let delegator_msa_id = 2u64;
		let page_id: PageId = 1;
		let schema_id = PAGINATED_SCHEMA;
		let caller: T::AccountId = whitelisted_caller();
		let payload = vec![0u8; s as usize];
		let schema_key = schema_id.encode().to_vec();

		assert_ok!(T::MsaBenchmarkHelper::add_key(provider_msa_id.into(), caller.clone()));
		assert_ok!(T::MsaBenchmarkHelper::set_delegation_relationship(provider_msa_id.into(), delegator_msa_id.into(), [PAGINATED_SCHEMA].to_vec()));

	}: _(RawOrigin::Signed(caller), delegator_msa_id.into(), schema_id, page_id, payload)
	verify {
		let page_result = StatefulStoragePallet::<T>::get_paginated_page(delegator_msa_id, schema_id, page_id);
		assert!(page_result.is_some());
		assert!(page_result.unwrap().data.len() > 0);
	}

	delete_page {
		let provider_msa_id = 1u64;
		let delegator_msa_id = 2u64;
		let schema_id = PAGINATED_SCHEMA;
		let page_id: PageId = 1;
		let caller: T::AccountId = whitelisted_caller();
		let payload = vec![0u8; T::MaxPaginatedPageSizeBytes::get() as usize];
		let schema_key = schema_id.encode().to_vec();

		T::SchemaBenchmarkHelper::set_schema_count(schema_id - 1);
		assert_ok!(create_schema::<T>(PayloadLocation::Paginated));
		assert_ok!(T::MsaBenchmarkHelper::add_key(provider_msa_id.into(), caller.clone()));
		assert_ok!(T::MsaBenchmarkHelper::set_delegation_relationship(provider_msa_id.into(), delegator_msa_id.into(), [schema_id].to_vec()));

		let page_key = page_id.encode().to_vec();
		StatefulChildTree::write(&delegator_msa_id, &[schema_key.clone(), page_key], payload.clone());
	}: _(RawOrigin::Signed(caller), delegator_msa_id.into(), schema_id, page_id)
	verify {
		let page_result = StatefulStoragePallet::<T>::get_paginated_page(delegator_msa_id, schema_id, page_id);
		assert!(page_result.is_none());
	}

	impl_benchmark_test_suite!(StatefulStoragePallet,
		crate::mock::new_test_ext(),
		crate::mock::Test);
}
