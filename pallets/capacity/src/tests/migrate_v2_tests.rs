#[cfg(test)]
mod test {
	use crate::{
		migration::{
			v2::v1::{
				StakingAccountDetails as OldStakingAccountDetails,
				StakingAccountLedger as OldStakingAccountLedger,
			},
			*,
		},
		tests::mock::*,
		types::*,
		BalanceOf, Config, StakingAccountLedger,
		StakingType::MaximumCapacity,
		UnstakeUnlocks,
	};
	use frame_support::{traits::StorageVersion, BoundedVec};

	#[test]
	#[allow(deprecated)]
	fn test_v1_to_v2_works() {
		new_test_ext().execute_with(|| {
			StorageVersion::new(1).put::<Capacity>();
			for i in 0..3u32 {
				let storage_key: <Test as frame_system::Config>::AccountId = i.into();
				let unlocks: BoundedVec<
					UnlockChunk<BalanceOf<Test>, <Test as Config>::EpochNumber>,
					<Test as Config>::MaxUnlockingChunks,
				> = BoundedVec::try_from(vec![
					UnlockChunk { value: i as u64, thaw_at: i + 10 },
					UnlockChunk { value: (i + 1) as u64, thaw_at: i + 20 },
					UnlockChunk { value: (i + 2) as u64, thaw_at: i + 30 },
				])
				.unwrap_or_default();

				let old_record =
					OldStakingAccountDetails::<Test> { active: 3, total: 5, unlocking: unlocks };
				OldStakingAccountLedger::<Test>::insert(storage_key, old_record);
			}

			assert_eq!(OldStakingAccountLedger::<Test>::iter().count(), 3);

			let _w = v2::migrate_to_v2::<Test>();

			assert_eq!(StakingAccountLedger::<Test>::iter().count(), 3);
			assert_eq!(UnstakeUnlocks::<Test>::iter().count(), 3);

			// check that this is really the new type
			let last_account: StakingDetails<Test> = Capacity::get_staking_account_for(2).unwrap();
			assert_eq!(last_account.staking_type, MaximumCapacity);

			let last_unlocks = Capacity::get_unstake_unlocking_for(2).unwrap();
			assert_eq!(last_unlocks.total(), 9u64);
		})
	}
}
