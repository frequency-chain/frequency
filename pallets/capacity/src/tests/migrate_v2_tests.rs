#[cfg(test)]
mod test {
	use crate::{
		migration::v2::{
			v1::{
				StakingAccountDetails as OldStakingAccountDetails,
				StakingAccountLedger as OldStakingAccountLedger,
			},
			MigrateToV2, STAKING_ID,
		},
		tests::mock::*,
		types::*,
		BalanceOf, Config, FreezeReason, StakingAccountLedger,
		StakingType::MaximumCapacity,
		UnstakeUnlocks,
	};
	use frame_support::{
		traits::{fungible::InspectFreeze, LockableCurrency, StorageVersion, WithdrawReasons},
		BoundedVec,
	};
	use pallet_balances::{BalanceLock, Reasons};

	type OldCurrency = pallet_balances::Pallet<Test>;

	#[test]
	#[allow(deprecated)]
	fn test_v1_to_v2_works<OldCurrency>()
	{
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

			// Create some Locks data in the old format
			// Grab an account with a balance
			let account = 200;
			let locked_amount = 50;

			pallet_balances::Pallet::<Test>::set_lock(
				STAKING_ID,
				&account,
				locked_amount,
				WithdrawReasons::all(),
			);

			// Confirm lock exists
			assert_eq!(
				pallet_balances::Pallet::<Test>::locks(&account).get(0),
				Some(&BalanceLock { id: STAKING_ID, amount: locked_amount, reasons: Reasons::All })
			);

			let test_record =
				StakingDetails::<Test> { active: locked_amount, staking_type: MaximumCapacity };
			StakingAccountLedger::<Test>::insert(account, test_record);
			assert_eq!(StakingAccountLedger::<Test>::iter().count(), 1);

			// Run migration.
			let _w = MigrateToV2::<Test, OldCurrency>::migrate_to_v2();

			assert_eq!(StakingAccountLedger::<Test>::iter().count(), 3);
			assert_eq!(UnstakeUnlocks::<Test>::iter().count(), 3);

			// check that this is really the new type
			let last_account: StakingDetails<Test> = Capacity::get_staking_account_for(2).unwrap();
			assert_eq!(last_account.staking_type, MaximumCapacity);

			let last_unlocks = Capacity::get_unstake_unlocking_for(2).unwrap();
			assert_eq!(9u64, unlock_chunks_total::<Test>(&last_unlocks));

			// Check that the old staking locks are now freezes
			assert_eq!(pallet_balances::Pallet::<Test>::locks(&account), vec![]);
			assert_eq!(
				<Test as Config>::Currency::balance_frozen(
					&FreezeReason::CapacityStaking.into(),
					&account
				),
				locked_amount
			);
		})
	}
}
