
#[cfg(test)]
mod test {
    use frame_support::{assert_noop, BoundedVec, Hashable};
    use super::*;
    use frame_support::traits::{ StorageVersion};
    use crate::{Config, StakingAccountLedger, types::*, tests::mock::*, migration::*, BalanceOf, UnstakeUnlocks};
    use crate::migration::v2::migration::{v1::StakingAccountLedger as OldStakingAccountLedger, v1::StakingAccountDetails as OldStakingAccountDetails};
    use crate::StakingType::MaximumCapacity;

    #[test]
    #[allow(deprecated)] // TODO: needed?
    fn test_v1_to_v2_works() {
        new_test_ext().execute_with(|| {
            StorageVersion::new(1).put::<Capacity>();
            for i in 0..3u32 {
                let storage_key: <Test as frame_system::Config>::AccountId = i.into();
                let unlocks: BoundedVec<UnlockChunk<BalanceOf<Test>, <Test as Config>::EpochNumber>, <Test as Config>::MaxUnlockingChunks> =
                    BoundedVec::try_from(vec![
                        UnlockChunk{ value: i as u64, thaw_at: i+10 },
                        UnlockChunk{ value: (i+1) as u64, thaw_at: i+20 },
                        UnlockChunk{ value: (i+2) as u64, thaw_at: i+30 },
                    ]).unwrap_or_default();

                let old_record = OldStakingAccountDetails::<BalanceOf<Test>, <Test as Config>::EpochNumber, <Test as Config>::MaxUnlockingChunks> {
                        active: 3, total: 5, unlocking: unlocks,
                    };
                // frame_support::migration::put_storage_value(b"Capacity", b"StakingAccountDetails", &storage_key, &old);
                OldStakingAccountLedger::<Test>::insert(storage_key, old_record);
            }

            assert_eq!(OldStakingAccountLedger::<Test>::iter().count(), 3);

            let _w = v2::migration::migrate_to_v2::<Test>();

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
