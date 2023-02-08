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
		assert!(page_result.data.len() > 0);
	}

	upsert_page {
		let n in 0 .. T::MaxPaginatedPageId::get() - 1;
		let s in 1 .. T::MaxPaginatedPageSizeBytes::get();
		let provider_msa_id = 1u64;
		let delegator_msa_id = 2u64;
		let schema_id = PAGINATED_SCHEMA;
		let caller: T::AccountId = whitelisted_caller();
		let payload = vec![0u8; s as usize];
		let schema_key = schema_id.encode().to_vec();

		assert_ok!(T::MsaBenchmarkHelper::add_key(provider_msa_id.into(), caller.clone()));
		assert_ok!(T::MsaBenchmarkHelper::set_delegation_relationship(provider_msa_id.into(), delegator_msa_id.into(), [PAGINATED_SCHEMA].to_vec()));

		for i in 0 .. n {
			let page_key = (n as PageId).encode().to_vec();
			StatefulChildTree::write(&delegator_msa_id, &[schema_key.clone(), page_key], payload.clone());
		}
		let page_id: u16 = (n + 1).try_into().unwrap();
	}: _(RawOrigin::Signed(caller), delegator_msa_id.into(), schema_id, page_id, payload)

	impl_benchmark_test_suite!(StatefulStoragePallet,
		crate::mock::new_test_ext(),
		crate::mock::Test);
}
