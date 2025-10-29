use crate::{
	migration::v1,
	stateful_child_tree::StatefulChildTree,
	test_common::constants::{UNDELEGATED_ITEMIZED_SCHEMA, UNDELEGATED_PAGINATED_SCHEMA},
	tests::mock::{
		new_test_ext, run_to_block_with_migrations, AllPalletsWithSystem, MigratorServiceWeight,
		System, Test as T,
	},
	types::{PageError, ITEMIZED_STORAGE_PREFIX, PAGINATED_STORAGE_PREFIX, PALLET_STORAGE_PREFIX},
	weights, Config,
};
use common_primitives::msa::MessageSourceId;
use frame_support::{
	assert_err, pallet_prelude::StorageVersion, traits::OnRuntimeUpgrade, BoundedVec,
};
use pallet_migrations::WeightInfo as _;
use parity_scale_codec::Decode;

fn check_paginated_pages<OkPageType: Decode, ErrPageType: Decode>(msa_ids: &Vec<MessageSourceId>) {
	for msa_id in msa_ids {
		for page_id in 1u16..<T as Config>::MaxPaginatedPageId::get() {
			let keys = (UNDELEGATED_PAGINATED_SCHEMA, page_id);
			assert!(StatefulChildTree::<<T as Config>::KeyHasher>::try_read::<_, ErrPageType>(
				&msa_id,
				PALLET_STORAGE_PREFIX,
				PAGINATED_STORAGE_PREFIX,
				&keys,
			)
			.is_err());
			assert!(StatefulChildTree::<<T as Config>::KeyHasher>::try_read::<_, OkPageType>(
				&msa_id,
				PALLET_STORAGE_PREFIX,
				PAGINATED_STORAGE_PREFIX,
				&keys,
			)
			.is_ok());
		}
	}
}

fn check_itemized_pages<OkPageType: Decode + core::fmt::Debug, ErrPageType: Decode>(
	msa_ids: &Vec<MessageSourceId>,
) -> Vec<OkPageType> {
	msa_ids
		.iter()
		.map(|msa_id| {
			let keys = (UNDELEGATED_ITEMIZED_SCHEMA,);
			assert!(StatefulChildTree::<<T as Config>::KeyHasher>::try_read::<_, ErrPageType>(
				&msa_id,
				PALLET_STORAGE_PREFIX,
				ITEMIZED_STORAGE_PREFIX,
				&keys,
			)
			.is_err());
			let page = StatefulChildTree::<<T as Config>::KeyHasher>::try_read::<_, OkPageType>(
				&msa_id,
				PALLET_STORAGE_PREFIX,
				ITEMIZED_STORAGE_PREFIX,
				&keys,
			);
			assert!(page.is_ok());
			page.unwrap().unwrap()
		})
		.collect()
}

#[test]
fn lazy_migration_works() {
	new_test_ext().execute_with(|| {
		const ITEMS_PER_BLOCK: u16 = 16;
		const MAX_BLOCKS: u16 = 50;
		let msa_ids = vec![1, 2, 3];
		frame_support::__private::sp_tracing::try_init_simple();

		// Insert some Paginated pages into storage.
		let mut page: v1::PaginatedPage<T> = v1::PaginatedPage::<T>::default();
		let payload: BoundedVec<u8, <T as Config>::MaxPaginatedPageSizeBytes> =
			vec![1; <T as Config>::MaxPaginatedPageSizeBytes::get() as usize]
				.try_into()
				.expect("Unable to create BoundedVec payload");
		page.data = payload;
		page.nonce = 1;
		for msa_id in &msa_ids {
			for page_id in 1u16..<T as Config>::MaxPaginatedPageId::get() {
				let keys: v1::PaginatedKey = (UNDELEGATED_PAGINATED_SCHEMA, page_id);
				StatefulChildTree::<<T as Config>::KeyHasher>::write(
					&msa_id,
					PALLET_STORAGE_PREFIX,
					PAGINATED_STORAGE_PREFIX,
					&keys,
					&page,
				);
			}
		}

		// Insert some Itemized pages into storage.
		let str: &[u8; 18] = b"This is a payload."; // length 18
		let mut page: v1::ItemizedPage<T> = v1::ItemizedPage::<T>::default();
		let payload =
			BoundedVec::<u8, <T as Config>::MaxItemizedBlobSizeBytes>::try_from(str.to_vec())
				.expect("Unable to create BoundedVec payload");
		let add_actions: Vec<v1::ItemAction<<T as Config>::MaxItemizedBlobSizeBytes>> =
			vec![0; 10usize]
				.iter()
				.map(|_| v1::ItemAction::Add { data: payload.clone() })
				.collect();
		let page = v1::ItemizedOperations::<T>::apply_item_actions(&mut page, &add_actions)
			.expect("failed to apply item actions");
		for msa_id in &msa_ids {
			let keys: v1::ItemizedKey = (UNDELEGATED_ITEMIZED_SCHEMA,);
			StatefulChildTree::<<T as Config>::KeyHasher>::write(
				&msa_id,
				PALLET_STORAGE_PREFIX,
				ITEMIZED_STORAGE_PREFIX,
				&keys,
				&page,
			);
		}

		// Pre-condition: NO pages should parse as V2; ALL pages should parse as V1
		check_paginated_pages::<v1::PaginatedPage<T>, crate::PaginatedPage<T>>(&msa_ids);
		let old_itemized_pages =
			check_itemized_pages::<v1::ItemizedPage<T>, crate::ItemizedPage<T>>(&msa_ids);
		old_itemized_pages.into_iter().for_each(|page| {
			// NO items should decode as V2; ALL items should decode as V1
			assert!(<v1::ItemizedPage<T> as v1::ItemizedOperations::<T>>::try_parse(&page).is_ok());

			// Convert the page payload to V2 and try to decode the items
			let page_v2 = crate::ItemizedPage::<T>::try_from(page.data).unwrap();
			assert_err!(
				<crate::ItemizedPage::<T> as crate::ItemizedOperations::<T>>::try_parse(
					&page_v2, true
				),
				PageError::ErrorParsing("decoding item header")
			);
		});

		// Give it a bunch of weight
		let limit = <T as pallet_migrations::Config>::WeightInfo::progress_mbms_none() +
			pallet_migrations::Pallet::<T>::exec_migration_max_weight() +
			<weights::SubstrateWeight<T> as crate::weights::WeightInfo>::paginated_v1_to_v2() *
				ITEMS_PER_BLOCK as u64;
		MigratorServiceWeight::set(&limit);

		StorageVersion::new(1).put::<crate::Pallet<T>>();
		System::set_block_number(1);
		AllPalletsWithSystem::on_runtime_upgrade(); // onboard MBMs

		for block in 2..=(MAX_BLOCKS + 2) {
			run_to_block_with_migrations(block as u32);
			if StorageVersion::new(2) == StorageVersion::get::<crate::Pallet<T>>() {
				break;
			}
		}

		// Make sure migration completed (ie, storage version was bumped)
		assert_eq!(StorageVersion::get::<crate::Pallet<T>>(), StorageVersion::new(2));

		// Check that all pages have been migrated
		check_paginated_pages::<crate::PaginatedPage<T>, v1::PaginatedPage<T>>(&msa_ids);
		let new_itemized_pages =
			check_itemized_pages::<crate::ItemizedPage<T>, v1::ItemizedPage<T>>(&msa_ids);
		new_itemized_pages.into_iter().for_each(|page| {
			// NO items should decode as V1; ALL items should decode as V2
			assert!(<crate::ItemizedPage<T> as crate::ItemizedOperations::<T>>::try_parse(
				&page, true
			)
			.is_ok());

			// Convert the page payload to V1 and try to decode the items
			let page_v1 = v1::ItemizedPage::<T>::try_from(page.data).unwrap();
			assert_err!(
				<v1::ItemizedPage::<T> as v1::ItemizedOperations::<T>>::try_parse(&page_v1),
				v1::PageError::ErrorParsing("wrong payload size")
			);
		});
	});
}
