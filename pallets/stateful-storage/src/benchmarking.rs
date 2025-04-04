#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use crate::{types::ItemAction, Pallet as StatefulStoragePallet};
use common_primitives::{
	schema::{ModelType, PayloadLocation},
	stateful_storage::{PageHash, PageId},
	utils::wrap_binary_data,
};
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::RawOrigin;
use parity_scale_codec::{Decode, Encode};
use sp_core::{bounded::BoundedVec, crypto::KeyTypeId};
use sp_runtime::RuntimeAppPublic;
use stateful_child_tree::StatefulChildTree;
use test_common::constants;
extern crate alloc;
use alloc::vec;

pub const TEST_KEY_TYPE_ID: KeyTypeId = KeyTypeId(*b"test");

mod app_sr25519 {
	use super::TEST_KEY_TYPE_ID;
	use sp_core::sr25519;
	use sp_runtime::app_crypto::app_crypto;
	app_crypto!(sr25519, TEST_KEY_TYPE_ID);
}

type SignerId = app_sr25519::Public;
pub const NONEXISTENT_PAGE_HASH: PageHash = 0;

fn itemized_actions_populate<T: Config>(
	n: u32,
	s: usize,
	delete_actions: u32,
) -> BoundedVec<ItemAction<T::MaxItemizedBlobSizeBytes>, T::MaxItemizedActionsCount> {
	let mut actions = vec![];
	for _ in 0..n {
		let payload = vec![0u8; s];
		actions.push(ItemAction::Add { data: payload.try_into().unwrap() });
	}
	for i in 0..delete_actions {
		actions.push(ItemAction::Delete { index: i as u16 });
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

fn get_itemized_page<T: Config>(
	msa_id: MessageSourceId,
	schema_id: SchemaId,
) -> Option<ItemizedPage<T>> {
	let key: ItemizedKey = (schema_id,);
	StatefulChildTree::<T::KeyHasher>::try_read::<_, ItemizedPage<T>>(
		&msa_id,
		PALLET_STORAGE_PREFIX,
		ITEMIZED_STORAGE_PREFIX,
		&key,
	)
	.unwrap_or(None)
}

fn get_paginated_page<T: Config>(
	msa_id: MessageSourceId,
	schema_id: SchemaId,
	page_id: PageId,
) -> Option<PaginatedPage<T>> {
	let key: PaginatedKey = (schema_id, page_id);
	StatefulChildTree::<T::KeyHasher>::try_read::<_, PaginatedPage<T>>(
		&msa_id,
		PALLET_STORAGE_PREFIX,
		PAGINATED_STORAGE_PREFIX,
		&key,
	)
	.unwrap_or(None)
}

benchmarks! {
	apply_item_actions_add {
		let s in (T::MaxItemizedBlobSizeBytes::get()) .. (T::MaxItemizedBlobSizeBytes::get() * T::MaxItemizedActionsCount::get());
		let provider_msa_id = 1u64;
		let delegator_msa_id = 2u64;
		let schema_id = constants::ITEMIZED_SCHEMA;
		let caller: T::AccountId = whitelisted_caller();
		let num_of_items = s / T::MaxItemizedBlobSizeBytes::get();
		let key = (schema_id,);

		T::SchemaBenchmarkHelper::set_schema_count(schema_id - 1);
		assert_ok!(create_schema::<T>(PayloadLocation::Itemized));
		assert_ok!(T::MsaBenchmarkHelper::add_key(provider_msa_id.into(), caller.clone()));
		assert_ok!(T::MsaBenchmarkHelper::set_delegation_relationship(provider_msa_id.into(), delegator_msa_id.into(), [schema_id].to_vec()));

		let actions = itemized_actions_populate::<T>(num_of_items, T::MaxItemizedBlobSizeBytes::get() as usize, 0);
	}: {
		assert_ok!(StatefulStoragePallet::<T>::apply_item_actions(RawOrigin::Signed(caller).into(), delegator_msa_id.into(), schema_id, NONEXISTENT_PAGE_HASH, actions));
	}
	verify {
		let page_result = get_itemized_page::<T>(delegator_msa_id, schema_id);
		assert!(page_result.is_some());
		assert!(page_result.unwrap().data.len() > 0);
	}

	apply_item_actions_delete {
		let n in 1 .. T::MaxItemizedActionsCount::get();
		let provider_msa_id = 1u64;
		let delegator_msa_id = 2u64;
		let schema_id = constants::ITEMIZED_SCHEMA;
		let caller: T::AccountId = whitelisted_caller();
		let num_of_items= n;
		// removed 2 bytes are for ItemHeader size which is currently 2 bytes per item
		let num_of_existing_items = T::MaxItemizedPageSizeBytes::get() / (T::MaxItemizedBlobSizeBytes::get() + 2);
		let key = (schema_id,);

		T::SchemaBenchmarkHelper::set_schema_count(schema_id - 1);
		assert_ok!(create_schema::<T>(PayloadLocation::Itemized));
		assert_ok!(T::MsaBenchmarkHelper::add_key(provider_msa_id.into(), caller.clone()));
		assert_ok!(T::MsaBenchmarkHelper::set_delegation_relationship(provider_msa_id.into(), delegator_msa_id.into(), [schema_id].to_vec()));

		for _ in 0..num_of_existing_items {
			let actions = itemized_actions_populate::<T>(1, T::MaxItemizedBlobSizeBytes::get() as usize, 0);
			let content_hash = StatefulChildTree::<T::KeyHasher>::try_read::<_, ItemizedPage::<T>>(
				&delegator_msa_id,
				PALLET_STORAGE_PREFIX,
				ITEMIZED_STORAGE_PREFIX,
				&key).unwrap().unwrap_or_default().get_hash();
			assert_ok!(StatefulStoragePallet::<T>::apply_item_actions(RawOrigin::Signed(caller.clone()).into(), delegator_msa_id.into(), schema_id, content_hash, actions));
		}

		let content_hash = StatefulChildTree::<T::KeyHasher>::try_read::<_, ItemizedPage::<T>>(
				&delegator_msa_id,
				PALLET_STORAGE_PREFIX,
				ITEMIZED_STORAGE_PREFIX,
				&key).unwrap().unwrap_or_default().get_hash();
		let actions = itemized_actions_populate::<T>(0, 0, num_of_items);
	}: {
		assert_ok!(StatefulStoragePallet::<T>::apply_item_actions(RawOrigin::Signed(caller).into(), delegator_msa_id.into(), schema_id, content_hash, actions));
	}
	verify {
		let page_result = get_itemized_page::<T>(delegator_msa_id, schema_id);
		assert!(page_result.is_some());
		assert!(page_result.unwrap().data.len() > 0);
	}

	upsert_page {
		let s in 1 .. T::MaxPaginatedPageSizeBytes::get();
		let provider_msa_id = 1u64;
		let delegator_msa_id = 2u64;
		let page_id: PageId = 1;
		let schema_id = constants::PAGINATED_SCHEMA;
		let caller: T::AccountId = whitelisted_caller();
		let payload = vec![1u8; s as usize];
		let max_payload = vec![1u8; T::MaxPaginatedPageSizeBytes::get() as usize];
		let page = PaginatedPage::<T>::from(BoundedVec::try_from(max_payload).unwrap());

		T::SchemaBenchmarkHelper::set_schema_count(schema_id - 1);
		assert_ok!(create_schema::<T>(PayloadLocation::Paginated));
		assert_ok!(T::MsaBenchmarkHelper::add_key(provider_msa_id.into(), caller.clone()));
		assert_ok!(T::MsaBenchmarkHelper::set_delegation_relationship(provider_msa_id.into(), delegator_msa_id.into(), [schema_id].to_vec()));

		let key = (schema_id, page_id);
		StatefulChildTree::<T::KeyHasher>::write(&delegator_msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&key, &page
		);
		let content_hash = StatefulChildTree::<T::KeyHasher>::try_read::<_, PaginatedPage::<T>>(
			&delegator_msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&key).expect("error reading").expect("no data").get_hash();
	}: _(RawOrigin::Signed(caller), delegator_msa_id.into(), schema_id, page_id, content_hash, payload.try_into().unwrap())
	verify {
		let page_result = get_paginated_page::<T>(delegator_msa_id, schema_id, page_id);
		assert!(page_result.is_some());
		assert!(page_result.unwrap().data.len() > 0);
	}

	delete_page {
		let provider_msa_id = 1u64;
		let delegator_msa_id = 2u64;
		let schema_id = constants::PAGINATED_SCHEMA;
		let page_id: PageId = 1;
		let caller: T::AccountId = whitelisted_caller();
		let payload = vec![0u8; T::MaxPaginatedPageSizeBytes::get() as usize];

		T::SchemaBenchmarkHelper::set_schema_count(schema_id - 1);
		assert_ok!(create_schema::<T>(PayloadLocation::Paginated));
		assert_ok!(T::MsaBenchmarkHelper::add_key(provider_msa_id.into(), caller.clone()));
		assert_ok!(T::MsaBenchmarkHelper::set_delegation_relationship(provider_msa_id.into(), delegator_msa_id.into(), [schema_id].to_vec()));

		let key = (schema_id, page_id);
		StatefulChildTree::<T::KeyHasher>::write(&delegator_msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&key, payload.clone()
		);
		let content_hash = StatefulChildTree::<T::KeyHasher>::try_read::<_, PaginatedPage::<T>>(
			&delegator_msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&key).unwrap().unwrap().get_hash();
	}: _(RawOrigin::Signed(caller), delegator_msa_id.into(), schema_id, page_id, content_hash)
	verify {
		let page_result = get_paginated_page::<T>(delegator_msa_id, schema_id, page_id);
		assert!(page_result.is_none());
	}

	apply_item_actions_with_signature_v2_add {
		let s in (T::MaxItemizedBlobSizeBytes::get()) .. (T::MaxItemizedBlobSizeBytes::get() * T::MaxItemizedActionsCount::get());
		let msa_id = 1u64;
		let schema_id = constants::ITEMIZED_SCHEMA;
		let caller: T::AccountId = whitelisted_caller();
		let num_of_items = s / T::MaxItemizedBlobSizeBytes::get();
		let key = (schema_id,);
		let expiration = BlockNumberFor::<T>::from(10u32);

		let delegator_account_public = SignerId::generate_pair(Some(constants::BENCHMARK_SIGNATURE_ACCOUNT_SEED.as_bytes().to_vec()));
		let delegator_account = T::AccountId::decode(&mut &delegator_account_public.encode()[..]).unwrap();
		let delegator_msa_id = constants::SIGNATURE_MSA_ID;

		T::SchemaBenchmarkHelper::set_schema_count(schema_id - 1);
		assert_ok!(create_schema::<T>(PayloadLocation::Itemized));
		assert_ok!(T::MsaBenchmarkHelper::add_key(msa_id.into(), caller.clone()));
		assert_ok!(T::MsaBenchmarkHelper::add_key(delegator_msa_id.into(), delegator_account.clone()));
		assert_ok!(T::MsaBenchmarkHelper::set_delegation_relationship(msa_id.into(), delegator_msa_id.into(), [schema_id].to_vec()));

		let actions = itemized_actions_populate::<T>(num_of_items, T::MaxItemizedBlobSizeBytes::get() as usize, 0);
		let payload = ItemizedSignaturePayloadV2 {
			actions,
			target_hash: NONEXISTENT_PAGE_HASH,
			expiration,
			schema_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let signature = delegator_account_public.sign(&encode_data_new_key_data).unwrap();
	}: {
		assert_ok!(StatefulStoragePallet::<T>::apply_item_actions_with_signature_v2(RawOrigin::Signed(caller).into(), delegator_account.into(), MultiSignature::Sr25519(signature.into()), payload));
	}
	verify {
		let page_result = get_itemized_page::<T>(delegator_msa_id, schema_id);
		assert!(page_result.is_some());
		assert!(page_result.unwrap().data.len() > 0);
	}

	apply_item_actions_with_signature_v2_delete {
		let n in 1 .. T::MaxItemizedActionsCount::get();
		let msa_id = 1u64;
		let schema_id = constants::ITEMIZED_SCHEMA;
		let caller: T::AccountId = whitelisted_caller();
		let num_of_items = n;
		let num_of_existing_items = T::MaxItemizedPageSizeBytes::get() / (T::MaxItemizedBlobSizeBytes::get() + 2);
		let key = (schema_id,);
		let expiration = BlockNumberFor::<T>::from(10u32);

		let delegator_account_public = SignerId::generate_pair(Some(constants::BENCHMARK_SIGNATURE_ACCOUNT_SEED.as_bytes().to_vec()));
		let delegator_account = T::AccountId::decode(&mut &delegator_account_public.encode()[..]).unwrap();
		let delegator_msa_id = constants::SIGNATURE_MSA_ID;

		T::SchemaBenchmarkHelper::set_schema_count(schema_id - 1);
		assert_ok!(create_schema::<T>(PayloadLocation::Itemized));
		assert_ok!(T::MsaBenchmarkHelper::add_key(msa_id.into(), caller.clone()));
		assert_ok!(T::MsaBenchmarkHelper::add_key(delegator_msa_id.into(), delegator_account.clone()));
		assert_ok!(T::MsaBenchmarkHelper::set_delegation_relationship(msa_id.into(), delegator_msa_id.into(), [schema_id].to_vec()));

		for _ in 0..num_of_existing_items {
			let actions = itemized_actions_populate::<T>(1, T::MaxItemizedBlobSizeBytes::get() as usize, 0);
			let content_hash = StatefulChildTree::<T::KeyHasher>::try_read::<_, ItemizedPage::<T>>(
				&delegator_msa_id,
				PALLET_STORAGE_PREFIX,
				ITEMIZED_STORAGE_PREFIX,
				&key).unwrap().unwrap_or_default().get_hash();
			assert_ok!(StatefulStoragePallet::<T>::apply_item_actions(RawOrigin::Signed(caller.clone()).into(), delegator_msa_id.into(), schema_id, content_hash, actions));
		}

		let content_hash = StatefulChildTree::<T::KeyHasher>::try_read::<_, ItemizedPage::<T>>(
				&delegator_msa_id,
				PALLET_STORAGE_PREFIX,
				ITEMIZED_STORAGE_PREFIX,
				&key).unwrap().unwrap_or_default().get_hash();
		let actions = itemized_actions_populate::<T>(0, 0, num_of_items);
		let payload = ItemizedSignaturePayloadV2 {
			actions,
			target_hash: content_hash,
			expiration,
			schema_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let signature = delegator_account_public.sign(&encode_data_new_key_data).unwrap();
	}: {
		assert_ok!(StatefulStoragePallet::<T>::apply_item_actions_with_signature_v2(RawOrigin::Signed(caller).into(), delegator_account.into(), MultiSignature::Sr25519(signature.into()), payload));
	}
	verify {
		let page_result = get_itemized_page::<T>(delegator_msa_id, schema_id);
		assert!(page_result.is_some());
		assert!(page_result.unwrap().data.len() > 0);
	}

	upsert_page_with_signature_v2 {
		let s in 1 .. T::MaxPaginatedPageSizeBytes::get();

		let provider_msa_id = 1u64;
		let delegator_msa_id = 2u64;
		let page_id: PageId = 1;
		let schema_id = constants::PAGINATED_SCHEMA;
		let caller: T::AccountId = whitelisted_caller();
		let payload = vec![0u8; s as usize];
		let max_payload = vec![1u8; T::MaxPaginatedPageSizeBytes::get() as usize];
		let page = PaginatedPage::<T>::from(BoundedVec::try_from(max_payload).unwrap());
		let schema_key = schema_id.encode().to_vec();
		let expiration = BlockNumberFor::<T>::from(10u32);

		let delegator_account_public = SignerId::generate_pair(Some(constants::BENCHMARK_SIGNATURE_ACCOUNT_SEED.as_bytes().to_vec()));
		let delegator_account = T::AccountId::decode(&mut &delegator_account_public.encode()[..]).unwrap();
		let delegator_msa_id = constants::SIGNATURE_MSA_ID;

		T::SchemaBenchmarkHelper::set_schema_count(schema_id - 1);
		assert_ok!(create_schema::<T>(PayloadLocation::Paginated));
		assert_ok!(T::MsaBenchmarkHelper::add_key(delegator_msa_id.into(), delegator_account.clone()));

		let key = (schema_id, page_id);
		StatefulChildTree::<T::KeyHasher>::write(&delegator_msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&key, &page
		);
		let content_hash = StatefulChildTree::<T::KeyHasher>::try_read::<_, PaginatedPage::<T>>(
			&delegator_msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&key).expect("error reading").expect("no data").get_hash();

		let payload = PaginatedUpsertSignaturePayloadV2 {
			payload: BoundedVec::try_from(payload).unwrap(),
			target_hash: content_hash,
			expiration,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let signature = delegator_account_public.sign(&encode_data_new_key_data).unwrap();
	}: _(RawOrigin::Signed(caller), delegator_account.into(), MultiSignature::Sr25519(signature.into()), payload)
	verify {
		let page_result = get_paginated_page::<T>(delegator_msa_id, schema_id, page_id);
		assert!(page_result.is_some());
		assert!(page_result.unwrap().data.len() > 0);
	}

	delete_page_with_signature_v2 {
		let provider_msa_id = 1u64;
		let delegator_msa_id = 2u64;
		let schema_id = constants::PAGINATED_SCHEMA;
		let page_id: PageId = 1;
		let caller: T::AccountId = whitelisted_caller();
		let payload = vec![0u8; T::MaxPaginatedPageSizeBytes::get() as usize];
		let expiration = BlockNumberFor::<T>::from(10u32);

		let delegator_account_public = SignerId::generate_pair(Some(constants::BENCHMARK_SIGNATURE_ACCOUNT_SEED.as_bytes().to_vec()));
		let delegator_account = T::AccountId::decode(&mut &delegator_account_public.encode()[..]).unwrap();
		let delegator_msa_id = constants::SIGNATURE_MSA_ID;

		T::SchemaBenchmarkHelper::set_schema_count(schema_id - 1);
		assert_ok!(create_schema::<T>(PayloadLocation::Paginated));
		assert_ok!(T::MsaBenchmarkHelper::add_key(delegator_msa_id.into(), delegator_account.clone()));

		let key = (schema_id, page_id);
		StatefulChildTree::<T::KeyHasher>::write(
			&delegator_msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&key,
			payload.clone(),
		);
		let content_hash = StatefulChildTree::<T::KeyHasher>::try_read::<_, PaginatedPage::<T>>(
			&delegator_msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&key).unwrap().unwrap().get_hash();

		let payload = PaginatedDeleteSignaturePayloadV2 {
			target_hash: content_hash,
			expiration,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let signature = delegator_account_public.sign(&encode_data_new_key_data).unwrap();
	}: _(RawOrigin::Signed(caller), delegator_account.into(), MultiSignature::Sr25519(signature.into()), payload)
	verify {
		let page_result = get_paginated_page::<T>(delegator_msa_id, schema_id, page_id);
		assert!(page_result.is_none());
	}

	impl_benchmark_test_suite!(StatefulStoragePallet,
		crate::tests::mock::new_test_ext_keystore(),
		crate::tests::mock::Test);
}
