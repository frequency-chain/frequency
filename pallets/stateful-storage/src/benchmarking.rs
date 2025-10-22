#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use crate::{types::ItemAction, Pallet as StatefulStoragePallet, StatefulChildTree};
use common_primitives::{
	schema::{ModelType, PayloadLocation},
	stateful_storage::{PageHash, PageId},
	utils::wrap_binary_data,
};
use frame_benchmarking::{v2::*, whitelisted_caller};
use frame_support::assert_ok;
use frame_system::RawOrigin;
use parity_scale_codec::{Decode, Encode};
use sp_core::{bounded::BoundedVec, crypto::KeyTypeId};
use sp_runtime::RuntimeAppPublic;
use test_common::constants;
extern crate alloc;
use alloc::vec;
use common_primitives::schema::IntentId;

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

fn create_intent_and_schema<T: Config>(location: PayloadLocation) -> Result<(), DispatchError> {
	let intent_id = T::SchemaBenchmarkHelper::create_intent(
		b"benchmark.test".to_vec(),
		location,
		Vec::default(),
	)?;
	let _ = T::SchemaBenchmarkHelper::create_schema(
		intent_id,
		Vec::from(r#"{"Message": "some-random-hash"}"#.as_bytes()),
		ModelType::AvroBinary,
		location,
	)?;
	Ok(())
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

fn get_itemized_page_v1<T: Config>(
	msa_id: MessageSourceId,
	schema_id: SchemaId,
) -> Option<migration::v1::ItemizedPage<T>> {
	let key: ItemizedKey = (schema_id,);
	StatefulChildTree::<T::KeyHasher>::try_read::<_, migration::v1::ItemizedPage<T>>(
		&msa_id,
		PALLET_STORAGE_PREFIX,
		ITEMIZED_STORAGE_PREFIX,
		&key,
	)
		.unwrap_or(None)
}

fn get_paginated_page_v1<T: Config>(
	msa_id: MessageSourceId,
	schema_id: SchemaId,
	page_id: PageId,
) -> Option<migration::v1::PaginatedPage<T>> {
	let key: migration::v1::PaginatedKey = (schema_id, page_id);
	StatefulChildTree::<T::KeyHasher>::try_read::<_, migration::v1::PaginatedPage<T>>(
		&msa_id,
		PALLET_STORAGE_PREFIX,
		PAGINATED_STORAGE_PREFIX,
		&key,
	)
	.unwrap_or(None)
}

fn get_paginated_page<T: Config>(
	msa_id: MessageSourceId,
	intent_id: IntentId,
	page_id: PageId,
) -> Option<PaginatedPage<T>> {
	let key: PaginatedKey = (intent_id, page_id);
	StatefulChildTree::<T::KeyHasher>::try_read::<_, PaginatedPage<T>>(
		&msa_id,
		PALLET_STORAGE_PREFIX,
		PAGINATED_STORAGE_PREFIX,
		&key,
	)
	.unwrap_or(None)
}

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn apply_item_actions_add(
		s: Linear<
			{ T::MaxItemizedBlobSizeBytes::get() },
			{ T::MaxItemizedBlobSizeBytes::get() * T::MaxItemizedActionsCount::get() },
		>,
	) -> Result<(), BenchmarkError> {
		let provider_msa_id = 1u64;
		let delegator_msa_id = 2u64;
		let schema_id = constants::ITEMIZED_SCHEMA;
		let caller: T::AccountId = whitelisted_caller();
		let num_of_items = s / T::MaxItemizedBlobSizeBytes::get();

		T::SchemaBenchmarkHelper::set_schema_count(schema_id - 1);
		assert_ok!(create_intent_and_schema::<T>(PayloadLocation::Itemized));
		assert_ok!(T::MsaBenchmarkHelper::add_key(provider_msa_id, caller.clone()));
		assert_ok!(T::MsaBenchmarkHelper::set_delegation_relationship(
			provider_msa_id.into(),
			delegator_msa_id.into(),
			[schema_id].to_vec()
		));

		let actions = itemized_actions_populate::<T>(
			num_of_items,
			T::MaxItemizedBlobSizeBytes::get() as usize,
			0,
		);
		#[block]
		{
			assert_ok!(StatefulStoragePallet::<T>::apply_item_actions(
				RawOrigin::Signed(caller).into(),
				delegator_msa_id,
				schema_id,
				NONEXISTENT_PAGE_HASH,
				actions
			));
		}

		let page_result = get_itemized_page::<T>(delegator_msa_id, schema_id);
		assert!(page_result.is_some());
		assert!(page_result.unwrap().data.len() > 0);
		Ok(())
	}

	#[benchmark]
	fn apply_item_actions_delete(
		n: Linear<1, { T::MaxItemizedActionsCount::get() }>,
	) -> Result<(), BenchmarkError> {
		let provider_msa_id = 1u64;
		let delegator_msa_id = 2u64;
		let schema_id = constants::ITEMIZED_SCHEMA;
		let caller: T::AccountId = whitelisted_caller();
		let num_of_items = n;
		// removed 2 bytes are for ItemHeader size which is currently 2 bytes per item
		let max_items = T::MaxItemizedPageSizeBytes::get() /
			(T::MaxItemizedBlobSizeBytes::get() + ItemHeader::max_encoded_len() as u32);
		let key = (schema_id,);

		T::SchemaBenchmarkHelper::set_schema_count(schema_id - 1);
		assert_ok!(create_intent_and_schema::<T>(PayloadLocation::Itemized));
		assert_ok!(T::MsaBenchmarkHelper::add_key(provider_msa_id, caller.clone()));
		assert_ok!(T::MsaBenchmarkHelper::set_delegation_relationship(
			provider_msa_id.into(),
			delegator_msa_id.into(),
			[schema_id].to_vec()
		));

		for _ in 0..max_items {
			let actions =
				itemized_actions_populate::<T>(1, T::MaxItemizedBlobSizeBytes::get() as usize, 0);
			let content_hash = StatefulChildTree::<T::KeyHasher>::try_read::<_, ItemizedPage<T>>(
				&delegator_msa_id,
				PALLET_STORAGE_PREFIX,
				ITEMIZED_STORAGE_PREFIX,
				&key,
			)
			.unwrap()
			.unwrap_or_default()
			.get_hash();
			assert_ok!(StatefulStoragePallet::<T>::apply_item_actions(
				RawOrigin::Signed(caller.clone()).into(),
				delegator_msa_id,
				schema_id,
				content_hash,
				actions
			));
		}

		let content_hash = StatefulChildTree::<T::KeyHasher>::try_read::<_, ItemizedPage<T>>(
			&delegator_msa_id,
			PALLET_STORAGE_PREFIX,
			ITEMIZED_STORAGE_PREFIX,
			&key,
		)
		.unwrap()
		.unwrap_or_default()
		.get_hash();
		let actions = itemized_actions_populate::<T>(0, 0, num_of_items);
		#[block]
		{
			assert_ok!(StatefulStoragePallet::<T>::apply_item_actions(
				RawOrigin::Signed(caller).into(),
				delegator_msa_id,
				schema_id,
				content_hash,
				actions
			));
		}

		let page_result = get_itemized_page::<T>(delegator_msa_id, schema_id);
		assert!(page_result.is_some());
		assert!(page_result.unwrap().data.len() > 0);
		Ok(())
	}

	#[benchmark]
	fn upsert_page(
		s: Linear<1, { T::MaxPaginatedPageSizeBytes::get() }>,
	) -> Result<(), BenchmarkError> {
		let provider_msa_id = 1u64;
		let delegator_msa_id = 2u64;
		let page_id: PageId = 1;
		let schema_id = constants::PAGINATED_SCHEMA;
		let caller: T::AccountId = whitelisted_caller();
		let payload = vec![1u8; s as usize];
		let max_payload = vec![1u8; T::MaxPaginatedPageSizeBytes::get() as usize];
		let page = PaginatedPage::<T>::from(BoundedVec::try_from(max_payload).unwrap());

		T::SchemaBenchmarkHelper::set_schema_count(schema_id - 1);
		assert_ok!(create_intent_and_schema::<T>(PayloadLocation::Paginated));
		assert_ok!(T::MsaBenchmarkHelper::add_key(provider_msa_id, caller.clone()));
		assert_ok!(T::MsaBenchmarkHelper::set_delegation_relationship(
			provider_msa_id.into(),
			delegator_msa_id.into(),
			[schema_id].to_vec()
		));

		let key = (schema_id, page_id);
		StatefulChildTree::<T::KeyHasher>::write(
			&delegator_msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&key,
			&page,
		);
		let content_hash = StatefulChildTree::<T::KeyHasher>::try_read::<_, PaginatedPage<T>>(
			&delegator_msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&key,
		)
		.expect("error reading")
		.expect("no data")
		.get_hash();

		#[extrinsic_call]
		_(
			RawOrigin::Signed(caller),
			delegator_msa_id,
			schema_id,
			page_id,
			content_hash,
			payload.try_into().unwrap(),
		);

		let page_result = get_paginated_page::<T>(delegator_msa_id, schema_id, page_id);
		assert!(page_result.is_some());
		assert!(page_result.unwrap().data.len() > 0);
		Ok(())
	}

	#[benchmark]
	fn delete_page() -> Result<(), BenchmarkError> {
		let provider_msa_id = 1u64;
		let delegator_msa_id = 2u64;
		let schema_id = constants::PAGINATED_SCHEMA;
		let intent_id = schema_id as IntentId;
		let page_id: PageId = 1;
		let caller: T::AccountId = whitelisted_caller();
		let payload = BoundedVec::<u8, T::MaxPaginatedPageSizeBytes>::try_from(vec![
			0u8;
			T::MaxPaginatedPageSizeBytes::get()
				as usize
		])
		.expect("failed to convert payload");
		let page = PaginatedPage::<T>::from(payload);

		T::SchemaBenchmarkHelper::set_schema_count(schema_id - 1);
		assert_ok!(create_intent_and_schema::<T>(PayloadLocation::Paginated));
		assert_ok!(T::MsaBenchmarkHelper::add_key(provider_msa_id, caller.clone()));
		assert_ok!(T::MsaBenchmarkHelper::set_delegation_relationship(
			provider_msa_id.into(),
			delegator_msa_id.into(),
			[intent_id].to_vec()
		));

		Pallet::<T>::update_paginated(delegator_msa_id, schema_id, page_id, 0, page)
			.expect("failed to write page");
		let key = (intent_id, page_id);
		let content_hash = StatefulChildTree::<T::KeyHasher>::try_read::<_, PaginatedPage<T>>(
			&delegator_msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&key,
		)
		.unwrap()
		.unwrap()
		.get_hash();

		#[extrinsic_call]
		_(RawOrigin::Signed(caller), delegator_msa_id, schema_id, page_id, content_hash);

		let page_result = get_paginated_page::<T>(delegator_msa_id, intent_id, page_id);
		assert!(page_result.is_none());
		Ok(())
	}

	#[benchmark]
	fn apply_item_actions_with_signature_v2_add(
		s: Linear<
			{ T::MaxItemizedBlobSizeBytes::get() },
			{ T::MaxItemizedBlobSizeBytes::get() * T::MaxItemizedActionsCount::get() },
		>,
	) -> Result<(), BenchmarkError> {
		let msa_id = 1u64;
		let schema_id = constants::ITEMIZED_SCHEMA;
		let caller: T::AccountId = whitelisted_caller();
		let num_of_items = s / T::MaxItemizedBlobSizeBytes::get();
		let expiration = BlockNumberFor::<T>::from(10u32);

		let delegator_account_public = SignerId::generate_pair(Some(
			constants::BENCHMARK_SIGNATURE_ACCOUNT_SEED.as_bytes().to_vec(),
		));
		let delegator_account =
			T::AccountId::decode(&mut &delegator_account_public.encode()[..]).unwrap();
		let delegator_msa_id = constants::SIGNATURE_MSA_ID;

		T::SchemaBenchmarkHelper::set_schema_count(schema_id - 1);
		assert_ok!(create_intent_and_schema::<T>(PayloadLocation::Itemized));
		assert_ok!(T::MsaBenchmarkHelper::add_key(msa_id, caller.clone()));
		assert_ok!(T::MsaBenchmarkHelper::add_key(delegator_msa_id, delegator_account.clone()));
		assert_ok!(T::MsaBenchmarkHelper::set_delegation_relationship(
			msa_id.into(),
			delegator_msa_id.into(),
			[schema_id].to_vec()
		));

		let actions = itemized_actions_populate::<T>(
			num_of_items,
			T::MaxItemizedBlobSizeBytes::get() as usize,
			0,
		);
		let payload = ItemizedSignaturePayloadV2 {
			actions,
			target_hash: NONEXISTENT_PAGE_HASH,
			expiration,
			schema_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let signature = delegator_account_public.sign(&encode_data_new_key_data).unwrap();
		#[block]
		{
			assert_ok!(StatefulStoragePallet::<T>::apply_item_actions_with_signature_v2(
				RawOrigin::Signed(caller).into(),
				delegator_account,
				MultiSignature::Sr25519(signature.into()),
				payload
			));
		}

		let page_result = get_itemized_page::<T>(delegator_msa_id, schema_id);
		assert!(page_result.is_some());
		assert!(page_result.unwrap().data.len() > 0);

		Ok(())
	}

	#[benchmark]
	fn apply_item_actions_with_signature_v2_delete(
		n: Linear<1, { T::MaxItemizedActionsCount::get() }>,
	) -> Result<(), BenchmarkError> {
		let msa_id = 1u64;
		let schema_id = constants::ITEMIZED_SCHEMA;
		let caller: T::AccountId = whitelisted_caller();
		let num_of_items = n;
		let num_of_existing_items = T::MaxItemizedPageSizeBytes::get() /
			(T::MaxItemizedBlobSizeBytes::get() + ItemHeader::max_encoded_len() as u32);
		let key = (schema_id,);
		let expiration = BlockNumberFor::<T>::from(10u32);

		let delegator_account_public = SignerId::generate_pair(Some(
			constants::BENCHMARK_SIGNATURE_ACCOUNT_SEED.as_bytes().to_vec(),
		));
		let delegator_account =
			T::AccountId::decode(&mut &delegator_account_public.encode()[..]).unwrap();
		let delegator_msa_id = constants::SIGNATURE_MSA_ID;

		T::SchemaBenchmarkHelper::set_schema_count(schema_id - 1);
		assert_ok!(create_intent_and_schema::<T>(PayloadLocation::Itemized));
		assert_ok!(T::MsaBenchmarkHelper::add_key(msa_id, caller.clone()));
		assert_ok!(T::MsaBenchmarkHelper::add_key(delegator_msa_id, delegator_account.clone()));
		assert_ok!(T::MsaBenchmarkHelper::set_delegation_relationship(
			msa_id.into(),
			delegator_msa_id.into(),
			[schema_id].to_vec()
		));

		for _ in 0..num_of_existing_items {
			let actions =
				itemized_actions_populate::<T>(1, T::MaxItemizedBlobSizeBytes::get() as usize, 0);
			let content_hash = StatefulChildTree::<T::KeyHasher>::try_read::<_, ItemizedPage<T>>(
				&delegator_msa_id,
				PALLET_STORAGE_PREFIX,
				ITEMIZED_STORAGE_PREFIX,
				&key,
			)
			.unwrap()
			.unwrap_or_default()
			.get_hash();
			assert_ok!(StatefulStoragePallet::<T>::apply_item_actions(
				RawOrigin::Signed(caller.clone()).into(),
				delegator_msa_id,
				schema_id,
				content_hash,
				actions
			));
		}

		let content_hash = StatefulChildTree::<T::KeyHasher>::try_read::<_, ItemizedPage<T>>(
			&delegator_msa_id,
			PALLET_STORAGE_PREFIX,
			ITEMIZED_STORAGE_PREFIX,
			&key,
		)
		.unwrap()
		.unwrap_or_default()
		.get_hash();
		let actions = itemized_actions_populate::<T>(0, 0, num_of_items);
		let payload = ItemizedSignaturePayloadV2 {
			actions,
			target_hash: content_hash,
			expiration,
			schema_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let signature = delegator_account_public.sign(&encode_data_new_key_data).unwrap();
		#[block]
		{
			assert_ok!(StatefulStoragePallet::<T>::apply_item_actions_with_signature_v2(
				RawOrigin::Signed(caller).into(),
				delegator_account,
				MultiSignature::Sr25519(signature.into()),
				payload
			));
		}

		let page_result = get_itemized_page::<T>(delegator_msa_id, schema_id);
		assert!(page_result.is_some());
		assert!(page_result.unwrap().data.len() > 0);
		Ok(())
	}

	#[benchmark]
	fn upsert_page_with_signature_v2(
		s: Linear<1, { T::MaxPaginatedPageSizeBytes::get() }>,
	) -> Result<(), BenchmarkError> {
		let page_id: PageId = 1;
		let schema_id = constants::PAGINATED_SCHEMA;
		let caller: T::AccountId = whitelisted_caller();
		let payload = vec![0u8; s as usize];
		let max_payload = vec![1u8; T::MaxPaginatedPageSizeBytes::get() as usize];
		let page = PaginatedPage::<T>::from(BoundedVec::try_from(max_payload).unwrap());
		let expiration = BlockNumberFor::<T>::from(10u32);

		let delegator_account_public = SignerId::generate_pair(Some(
			constants::BENCHMARK_SIGNATURE_ACCOUNT_SEED.as_bytes().to_vec(),
		));
		let delegator_account =
			T::AccountId::decode(&mut &delegator_account_public.encode()[..]).unwrap();
		let delegator_msa_id = constants::SIGNATURE_MSA_ID;

		T::SchemaBenchmarkHelper::set_schema_count(schema_id - 1);
		assert_ok!(create_intent_and_schema::<T>(PayloadLocation::Paginated));
		assert_ok!(T::MsaBenchmarkHelper::add_key(delegator_msa_id, delegator_account.clone()));

		let key = (schema_id, page_id);
		StatefulChildTree::<T::KeyHasher>::write(
			&delegator_msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&key,
			&page,
		);
		let content_hash = StatefulChildTree::<T::KeyHasher>::try_read::<_, PaginatedPage<T>>(
			&delegator_msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&key,
		)
		.expect("error reading")
		.expect("no data")
		.get_hash();

		let payload = PaginatedUpsertSignaturePayloadV2 {
			payload: BoundedVec::try_from(payload).unwrap(),
			target_hash: content_hash,
			expiration,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let signature = delegator_account_public.sign(&encode_data_new_key_data).unwrap();

		#[extrinsic_call]
		_(
			RawOrigin::Signed(caller),
			delegator_account,
			MultiSignature::Sr25519(signature.into()),
			payload,
		);

		let page_result = get_paginated_page::<T>(delegator_msa_id, schema_id, page_id);
		assert!(page_result.is_some());
		assert!(page_result.unwrap().data.len() > 0);
		Ok(())
	}

	#[benchmark]
	fn delete_page_with_signature_v2() -> Result<(), BenchmarkError> {
		let schema_id = constants::PAGINATED_SCHEMA;
		let page_id: PageId = 1;
		let caller: T::AccountId = whitelisted_caller();
		let payload = BoundedVec::<u8, T::MaxPaginatedPageSizeBytes>::try_from(vec![
			0u8;
			T::MaxPaginatedPageSizeBytes::get()
				as usize
		])
		.expect("failed to convert payload");
		let page = PaginatedPage::<T>::from(payload);
		let expiration = BlockNumberFor::<T>::from(10u32);

		let delegator_account_public = SignerId::generate_pair(Some(
			constants::BENCHMARK_SIGNATURE_ACCOUNT_SEED.as_bytes().to_vec(),
		));
		let delegator_account =
			T::AccountId::decode(&mut &delegator_account_public.encode()[..]).unwrap();
		let delegator_msa_id = constants::SIGNATURE_MSA_ID;

		T::SchemaBenchmarkHelper::set_schema_count(schema_id - 1);
		assert_ok!(create_intent_and_schema::<T>(PayloadLocation::Paginated));
		assert_ok!(T::MsaBenchmarkHelper::add_key(delegator_msa_id, delegator_account.clone()));

		let key = (schema_id, page_id);
		Pallet::<T>::update_paginated(delegator_msa_id, schema_id, page_id, 0, page)
			.expect("failed to write page");
		let content_hash = StatefulChildTree::<T::KeyHasher>::try_read::<_, PaginatedPage<T>>(
			&delegator_msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&key,
		)
		.unwrap()
		.unwrap()
		.get_hash();

		let payload = PaginatedDeleteSignaturePayloadV2 {
			target_hash: content_hash,
			expiration,
			schema_id,
			page_id,
		};
		let encode_data_new_key_data = wrap_binary_data(payload.encode());
		let signature = delegator_account_public.sign(&encode_data_new_key_data).unwrap();

		#[extrinsic_call]
		_(
			RawOrigin::Signed(caller),
			delegator_account,
			MultiSignature::Sr25519(signature.into()),
			payload,
		);

		let page_result = get_paginated_page::<T>(delegator_msa_id, schema_id, page_id);
		assert!(page_result.is_none());
		Ok(())
	}

	#[benchmark]
	fn paginated_v1_to_v2() -> Result<(), BenchmarkError> {
		// Setup
		let msa_id: MessageSourceId = T::MsaBenchmarkHelper::create_msa(whitelisted_caller())?;
		let schema_id: SchemaId = 1;
		let page_id: PageId = 1;

		// Create a paginated page
		let mut page: migration::v1::PaginatedPage<T> =
			migration::v1::PaginatedPage::<T>::default();
		let payload: BoundedVec<u8, T::MaxPaginatedPageSizeBytes> =
			vec![1; T::MaxPaginatedPageSizeBytes::get() as usize]
				.try_into()
				.expect("Unable to create BoundedVec payload");
		page.data = payload;
		page.nonce = 1;
		let keys: migration::v1::PaginatedKey = (schema_id, page_id);
		StatefulChildTree::<T::KeyHasher>::write(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
			&keys,
			&page,
		);
		let created_page = get_paginated_page_v1::<T>(msa_id, schema_id, page_id);
		assert!(created_page.is_some());

		let child = StatefulChildTree::<T::KeyHasher>::get_child_tree_for_storage(
			msa_id,
			PALLET_STORAGE_PREFIX,
			PAGINATED_STORAGE_PREFIX,
		);
		let mut cursor = migration::v2::ChildCursor::<migration::v2::PaginatedKeyLength> {
			id: msa_id,
			last_key: BoundedVec::default(),
		};

		// Execute
		#[block]
		{
			migration::v2::process_paginated_page::<
				T,
				migration::v2::PaginatedKeyLength,
			>(&child, &mut cursor)
			.expect("failed to migrate paginated page");
		}

		let updated_page = get_paginated_page::<T>(msa_id, schema_id, page_id);
		assert!(updated_page.is_some());
		let updated_page = updated_page.unwrap();
		assert_eq!(updated_page.page_version, PageVersion::V2);
		assert_eq!(updated_page.schema_id, Some(schema_id));

		Ok(())
	}

	#[benchmark]
	fn itemized_v1_to_v2() -> Result<(), BenchmarkError> {
		// Setup
		let msa_id: MessageSourceId = T::MsaBenchmarkHelper::create_msa(whitelisted_caller())?;
		let schema_id: SchemaId = 1;

		// Construct a page that represents the worst case across Testnet + Mainnet.
		// Current page size limit there is (10 * (1024 + 2))= 10260
		// The most full Itemized page is < 2% of the max page size, ~200 bytes
		// The largest single item is 33 bytes.
		// This likely represnts much more fullness in the test case (~20% vs ~2%), but as long as
		// we are under 25% fullness we are guaranteed to be able to migrate all pages

		// Compute item size for 10 items totalling 20% fullness
		// (page_size / 10 / 5 ) = 1026 / 10 / 6 = 18
		let max_items = 10;
		let str: &[u8; 18] = b"This is a payload."; // length 18

		// Create a paginated page
		let mut page: migration::v1::ItemizedPage<T> =
			migration::v1::ItemizedPage::<T>::default();
		let payload = BoundedVec::<u8, T::MaxItemizedBlobSizeBytes>::try_from(str.to_vec()).expect("Unable to create BoundedVec payload");
		let add_actions: Vec<migration::v1::ItemAction<T::MaxItemizedBlobSizeBytes>> = vec![0; max_items as usize].iter().map(|_| migration::v1::ItemAction::Add { data: payload.clone() }).collect();
		let page = migration::v1::ItemizedOperations::<T>::apply_item_actions(&mut page, &add_actions).expect("failed to apply item actions");
		let keys: migration::v1::ItemizedKey = (schema_id,);
		StatefulChildTree::<T::KeyHasher>::write(
			&msa_id,
			PALLET_STORAGE_PREFIX,
			ITEMIZED_STORAGE_PREFIX,
			&keys,
			&page,
		);
		let created_page = get_itemized_page_v1::<T>(msa_id, schema_id);
		assert!(created_page.is_some());
		let created_page = created_page.unwrap();
		let orig_parsed_page = migration::v1::ItemizedOperations::<T>::try_parse(&created_page).expect("unable to parse newly-written page");
		assert_eq!(orig_parsed_page.items.len(), max_items as usize);

		let child = StatefulChildTree::<T::KeyHasher>::get_child_tree_for_storage(
			msa_id,
			PALLET_STORAGE_PREFIX,
			ITEMIZED_STORAGE_PREFIX,
		);
		let mut cursor = migration::v2::ChildCursor::<migration::v2::ItemizedKeyLength> {
			id: msa_id,
			last_key: BoundedVec::default(),
		};

		// Execute
		#[block]
		{
			migration::v2::process_itemized_page::<
				T,
				migration::v2::ItemizedKeyLength,
			>(&child, &mut cursor)
				.expect("failed to migrate itemized page");
		}

		let updated_page = get_itemized_page::<T>(msa_id, schema_id);
		assert!(updated_page.is_some());
		let updated_page = updated_page.unwrap();
		assert_eq!(updated_page.page_version, PageVersion::V2);
		assert_eq!(updated_page.schema_id, None);
		assert_eq!(updated_page.nonce, created_page.nonce + 1);
		let updated_page = crate::ItemizedOperations::<T>::try_parse(&updated_page, false);
		assert!(updated_page.is_ok());
		let updated_page = updated_page.unwrap();
		assert_eq!(updated_page.items.len(), max_items as usize);
		updated_page.items.iter().for_each(|(i, item)| {
			let orig_page = orig_parsed_page.items.get(i);
			assert!(orig_page.is_some());
			let orig_page = orig_page.unwrap();
			assert_eq!(*item, orig_page.data.as_slice());
			assert_eq!(*item, str);
		});

		Ok(())
	}

	impl_benchmark_test_suite!(
		StatefulStoragePallet,
		crate::tests::mock::new_test_ext_keystore(),
		crate::tests::mock::Test
	);
}
