use frame_support::BoundedVec;
use crate::{migration::{
	v2,
}, pallet, tests::mock::{
	new_test_ext, run_to_block_with_migrations, AllPalletsWithSystem, MigratorServiceWeight, Test as T,
	System,
}, weights};
use frame_support::{pallet_prelude::StorageVersion, traits::OnRuntimeUpgrade};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_migrations::WeightInfo as _;

#[test]
fn lazy_migration_works() {
	new_test_ext().execute_with(|| {
		const MESSAGE_COUNT: u16 = 500;
		const ITEMS_PER_BLOCK: u16 = 16;
		const BLOCKS_PER_RUN: u16 = MESSAGE_COUNT.div_ceil(ITEMS_PER_BLOCK);
		frame_support::__private::sp_tracing::try_init_simple();
		// Insert some values into the old storage map.
		let payload: BoundedVec<u8, <T as pallet::Config>::MessagesMaxPayloadSizeBytes> =
			vec![1; 3072].try_into().expect("Unable to create BoundedVec payload");
		let old_message = v2::Message {
			payload: payload.clone(),
			provider_msa_id: 1u64,
			msa_id: Some(1u64),
		};
		let mut keys: Vec<(BlockNumberFor<T>, u16, u16)> = Vec::new();
		for i in 0..MESSAGE_COUNT {
			keys.push((BlockNumberFor::<T>::default(), 0u16, i));
			v2::MessagesV2::<T>::insert(
				(BlockNumberFor::<T>::default(), 0u16, i),
				old_message.clone(),
			);
		}

		// Give it enough weight do do exactly 16 iterations:
		let limit = <T as pallet_migrations::Config>::WeightInfo::progress_mbms_none() +
			pallet_migrations::Pallet::<T>::exec_migration_max_weight() +
			<weights::SubstrateWeight<T> as crate::weights::WeightInfo>::v2_to_v3_step() * ITEMS_PER_BLOCK as u64;
		MigratorServiceWeight::set(&limit);

		StorageVersion::new(2).put::<crate::Pallet<T>>();
		System::set_block_number(1);
		AllPalletsWithSystem::on_runtime_upgrade(); // onboard MBMs

		let mut v2_expected: u16 = 500;
		let mut v3_expected: u16 = 0;
		for block in 2..=(BLOCKS_PER_RUN + 2) {
			println!("Block: {}", block);
			run_to_block_with_migrations(block as u32);

			v2_expected = v2_expected.saturating_sub(ITEMS_PER_BLOCK);
			v3_expected += ITEMS_PER_BLOCK;
			v3_expected = u16::min(v3_expected, MESSAGE_COUNT);

			let v2_count = v2::MessagesV2::<T>::iter().count();
			let v3_count = crate::MessagesV3::<T>::iter().count();
			assert_eq!(v2_expected as usize, v2_count);
			assert_eq!(v3_expected as usize, v3_count);
		}

		// Check that everything is in new storage now:
		for i in keys.iter() {
			assert!(crate::MessagesV3::<T>::contains_key(i));
		}

		// Verify that the version is updated:
		let current_version = StorageVersion::get::<crate::Pallet<T>>();
		assert_eq!(current_version, 3);
	});
}
