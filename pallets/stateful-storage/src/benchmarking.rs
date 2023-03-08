#![allow(unused_must_use)]

use super::*;
use crate::{types::ItemAction, Pallet as StatefulStoragePallet};
use codec::{Decode, Encode};
use common_primitives::{
	schema::{ModelType, PayloadLocation},
	stateful_storage::{PageHash, PageId},
};
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::RawOrigin;
use sp_core::{bounded::BoundedVec, crypto::KeyTypeId};
use sp_runtime::RuntimeAppPublic;
use stateful_child_tree::StatefulChildTree;
use test_common::*;

pub const TEST_KEY_TYPE_ID: KeyTypeId = KeyTypeId(*b"test");

mod app_sr25519 {
	use super::TEST_KEY_TYPE_ID;
	use sp_core::sr25519;
	use sp_runtime::app_crypto::app_crypto;
	app_crypto!(sr25519, TEST_KEY_TYPE_ID);
}

type SignerId = app_sr25519::Public;
pub const NONEXISTENT_PAGE_HASH: PageHash = 0;

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
		let p in 1 .. T::MaxItemizedPageSizeBytes::get()- 1;
		let provider_msa_id = 1u64;
		let delegator_msa_id = 2u64;
		let schema_id = constants::ITEMIZED_SCHEMA;
		let caller: T::AccountId = whitelisted_caller();
		let payload = vec![0u8; s as usize];
		let num_of_items = p / (T::MaxItemizedPageSizeBytes::get() + 2);
		let key = (schema_id,);

		T::SchemaBenchmarkHelper::set_schema_count(schema_id - 1);
		assert_ok!(create_schema::<T>(PayloadLocation::Itemized));
		assert_ok!(T::MsaBenchmarkHelper::add_key(provider_msa_id.into(), caller.clone()));
		assert_ok!(T::MsaBenchmarkHelper::set_delegation_relationship(provider_msa_id.into(), delegator_msa_id.into(), [schema_id].to_vec()));

		for _ in 0..num_of_items {
			let actions = itemized_actions_add::<T>(1, T::MaxItemizedBlobSizeBytes::get() as usize);
			let content_hash = StatefulChildTree::<T::KeyHasher>::try_read::<_, ItemizedPage::<T>>(
				&delegator_msa_id,
				PALLET_STORAGE_PREFIX,
				ITEMIZED_STORAGE_PREFIX,
				&key).unwrap().unwrap_or_default().get_hash();
			assert_ok!(StatefulStoragePallet::<T>::apply_item_actions(RawOrigin::Signed(caller.clone()).into(), delegator_msa_id.into(), schema_id, content_hash, actions));
		}

		let actions = itemized_actions_add::<T>(n, s as usize);
	}: {
		// Explicity call SignedExtension checks because overhead benchmark will not trigger them. Map the returned error to any valid DispatchError.
		let _ = StatefulStoragePallet::<T>::validate_itemized(&delegator_msa_id, &schema_id, &actions, false, &NONEXISTENT_PAGE_HASH).map_err(|_| -> DispatchError {Error::<T>::CorruptedState.into()})?;
		StatefulStoragePallet::<T>::apply_item_actions(RawOrigin::Signed(caller).into(), delegator_msa_id.into(), schema_id, NONEXISTENT_PAGE_HASH, actions)
	}
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
		let schema_id = constants::PAGINATED_SCHEMA;
		let caller: T::AccountId = whitelisted_caller();
		let payload = vec![0u8; s as usize];
		let schema_key = schema_id.encode().to_vec();

		T::SchemaBenchmarkHelper::set_schema_count(schema_id - 1);
		assert_ok!(create_schema::<T>(PayloadLocation::Paginated));
		assert_ok!(T::MsaBenchmarkHelper::add_key(provider_msa_id.into(), caller.clone()));
		assert_ok!(T::MsaBenchmarkHelper::set_delegation_relationship(provider_msa_id.into(), delegator_msa_id.into(), [schema_id].to_vec()));
	}: {
		// Explicity call SignedExtension checks because overhead benchmark will not trigger them. Map the returned error to any valid DispatchError.
		let _ = StatefulStoragePallet::<T>::validate_paginated(&delegator_msa_id, &schema_id, &page_id, false, false, &NONEXISTENT_PAGE_HASH).map_err(|_| -> DispatchError {Error::<T>::CorruptedState.into()})?;
		StatefulStoragePallet::<T>::upsert_page(RawOrigin::Signed(caller).into(), delegator_msa_id.into(), schema_id, page_id, NONEXISTENT_PAGE_HASH, payload.try_into().unwrap())
	}
	verify {
		let page_result = StatefulStoragePallet::<T>::get_paginated_page(delegator_msa_id, schema_id, page_id);
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
	}: {
		// Explicity call SignedExtension checks because overhead benchmark will not trigger them. Map the returned error to any valid DispatchError.
		let _ = StatefulStoragePallet::<T>::validate_paginated(&delegator_msa_id, &schema_id, &page_id, false, true, &content_hash).map_err(|_| -> DispatchError {Error::<T>::CorruptedState.into()})?;
		StatefulStoragePallet::<T>::delete_page(RawOrigin::Signed(caller).into(), delegator_msa_id.into(), schema_id, page_id, content_hash)
	}
	verify {
		let page_result = StatefulStoragePallet::<T>::get_paginated_page(delegator_msa_id, schema_id, page_id);
		assert!(page_result.is_none());
	}

	apply_item_actions_with_signature {
		let n in 1 .. T::MaxItemizedActionsCount::get() - 1;
		let s in 1 .. T::MaxItemizedBlobSizeBytes::get()- 1;

		let msa_id = 1u64;
		let schema_id = constants::ITEMIZED_SCHEMA;
		let caller: T::AccountId = whitelisted_caller();
		let payload = vec![0u8; s as usize];
		let key = (schema_id,);
		let expiration = <T as frame_system::Config>::BlockNumber::from(10u32);

		let delegator_account_public = SignerId::generate_pair(Some(constants::BENCHMARK_SIGNATURE_ACCOUNT_SEED.as_bytes().to_vec()));
		let delegator_account = T::AccountId::decode(&mut &delegator_account_public.encode()[..]).unwrap();
		let delegator_msa_id = constants::SIGNATURE_MSA_ID;

		T::SchemaBenchmarkHelper::set_schema_count(schema_id - 1);
		assert_ok!(create_schema::<T>(PayloadLocation::Itemized));
		assert_ok!(T::MsaBenchmarkHelper::add_key(delegator_msa_id.into(), delegator_account.clone()));

		let actions = itemized_actions_add::<T>(n, s as usize);
		let payload = ItemizedSignaturePayload {
			actions,
			target_hash: PageHash::default(),
			msa_id: delegator_msa_id,
			expiration,
			schema_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let signature = delegator_account_public.sign(&encode_data_new_key_data).unwrap();
	}: {
		// Explicity call SignedExtension checks because overhead benchmark will not trigger them. Map the returned error to any valid DispatchError.
		let _ = StatefulStoragePallet::<T>::validate_itemized(&payload.msa_id, &payload.schema_id, &payload.actions, true, &payload.target_hash).map_err(|_| -> DispatchError {Error::<T>::CorruptedState.into()})?;
		StatefulStoragePallet::<T>::apply_item_actions_with_signature(RawOrigin::Signed(caller).into(), delegator_account.into(), MultiSignature::Sr25519(signature.into()), payload)
	}
	verify {
		let page_result = StatefulStoragePallet::<T>::get_itemized_page(delegator_msa_id, schema_id);
		assert!(page_result.is_some());
		assert!(page_result.unwrap().data.len() > 0);
	}

	upsert_page_with_signature {
		let s in 1 .. T::MaxPaginatedPageSizeBytes::get();

		let provider_msa_id = 1u64;
		let delegator_msa_id = 2u64;
		let page_id: PageId = 1;
		let schema_id = constants::PAGINATED_SCHEMA;
		let caller: T::AccountId = whitelisted_caller();
		let payload = vec![0u8; s as usize];
		let schema_key = schema_id.encode().to_vec();
		let expiration = <T as frame_system::Config>::BlockNumber::from(10u32);

		let delegator_account_public = SignerId::generate_pair(Some(constants::BENCHMARK_SIGNATURE_ACCOUNT_SEED.as_bytes().to_vec()));
		let delegator_account = T::AccountId::decode(&mut &delegator_account_public.encode()[..]).unwrap();
		let delegator_msa_id = constants::SIGNATURE_MSA_ID;

		T::SchemaBenchmarkHelper::set_schema_count(schema_id - 1);
		assert_ok!(create_schema::<T>(PayloadLocation::Paginated));
		assert_ok!(T::MsaBenchmarkHelper::add_key(delegator_msa_id.into(), delegator_account.clone()));

		let payload = PaginatedUpsertSignaturePayload {
			payload: BoundedVec::try_from(payload).unwrap(),
			target_hash: PageHash::default(),
			msa_id: delegator_msa_id,
			expiration,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let signature = delegator_account_public.sign(&encode_data_new_key_data).unwrap();
	}: {
		// Explicity call SignedExtension checks because overhead benchmark will not trigger them. Map the returned error to any valid DispatchError.
		let _ = StatefulStoragePallet::<T>::validate_paginated(&payload.msa_id, &payload.schema_id, &payload.page_id, true, false, &payload.target_hash).map_err(|_| -> DispatchError {Error::<T>::CorruptedState.into()})?;
		StatefulStoragePallet::<T>::upsert_page_with_signature(RawOrigin::Signed(caller).into(), delegator_account.into(), MultiSignature::Sr25519(signature.into()), payload)
	}
	verify {
		let page_result = StatefulStoragePallet::<T>::get_paginated_page(delegator_msa_id, schema_id, page_id);
		assert!(page_result.is_some());
		assert!(page_result.unwrap().data.len() > 0);
	}

	delete_page_with_signature {
		let provider_msa_id = 1u64;
		let delegator_msa_id = 2u64;
		let schema_id = constants::PAGINATED_SCHEMA;
		let page_id: PageId = 1;
		let caller: T::AccountId = whitelisted_caller();
		let payload = vec![0u8; T::MaxPaginatedPageSizeBytes::get() as usize];
		let expiration = <T as frame_system::Config>::BlockNumber::from(10u32);

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

		let payload = PaginatedDeleteSignaturePayload {
			target_hash: content_hash,
			msa_id: delegator_msa_id,
			expiration,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let signature = delegator_account_public.sign(&encode_data_new_key_data).unwrap();
	}: {
		// Explicity call SignedExtension checks because overhead benchmark will not trigger them. Map the returned error to any valid DispatchError.
		let _ = StatefulStoragePallet::<T>::validate_paginated(&payload.msa_id, &payload.schema_id, &payload.page_id, true, true, &payload.target_hash).map_err(|_| -> DispatchError {Error::<T>::CorruptedState.into()})?;
		StatefulStoragePallet::<T>::delete_page_with_signature(RawOrigin::Signed(caller).into(), delegator_account.into(), MultiSignature::Sr25519(signature.into()), payload)
	}
	verify {
		let page_result = StatefulStoragePallet::<T>::get_paginated_page(delegator_msa_id, schema_id, page_id);
		assert!(page_result.is_none());
	}

	impl_benchmark_test_suite!(StatefulStoragePallet,
		crate::mock::new_test_ext_keystore(),
		crate::mock::Test);
}
