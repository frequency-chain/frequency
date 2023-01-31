use super::*;
use crate as pallet_capacity;
use crate::mock::*;
use frame_support::BoundedVec;

#[test]
fn staking_account_details_reap_thawed_happy_path() {
	let mut staking_account = StakingAccountDetails::<Test>::default();
	staking_account.increase_by(10);

	// 10 token total, 6 token unstaked
	let new_unlocks: Vec<(u32, u32)> = vec![(1u32, 2u32), (2u32, 3u32), (3u32, 4u32)];
	assert_eq!(true, staking_account.set_unlock_chunks(&new_unlocks));
	assert_eq!(10, staking_account.total);
	assert_eq!(3, staking_account.unlocking.len());

	// At epoch 3, the first two chunks should be thawed.
	assert_eq!(3u64, staking_account.reap_thawed(3u32));
	assert_eq!(1, staking_account.unlocking.len());
	// ...leaving 10-3 = 7 total in staking
	assert_eq!(7, staking_account.total);

	// At epoch 5, all unstaking is done.
	assert_eq!(3u64, staking_account.reap_thawed(5u32));
	assert_eq!(0, staking_account.unlocking.len());
	// ...leaving 7-3 = 4 total
	assert_eq!(4, staking_account.total);
}

#[test]
fn impl_staking_account_details_increase_by() {
	new_test_ext().execute_with(|| {
		let mut staking_account = StakingAccountDetails::<Test>::default();
		assert_eq!(staking_account.increase_by(10), Some(()));

		assert_eq!(
			staking_account,
			StakingAccountDetails::<Test> {
				active: BalanceOf::<Test>::from(10u64),
				total: BalanceOf::<Test>::from(10u64),
				unlocking: BoundedVec::default(),
			}
		)
	});
}

#[test]
fn impl_staking_account_details_default() {
	new_test_ext().execute_with(|| {
		assert_eq!(
			StakingAccountDetails::<Test>::default(),
			StakingAccountDetails::<Test> {
				active: BalanceOf::<Test>::zero(),
				total: BalanceOf::<Test>::zero(),
				unlocking: BoundedVec::default(),
			},
		);
	});
}

#[test]
fn impl_staking_account_details_get_stakable_amount_for() {
	new_test_ext().execute_with(|| {
		let account = 100;
		let staking_account = StakingAccountDetails::<Test>::default();

		// When staking all of free balance.
		assert_eq!(staking_account.get_stakable_amount_for(&account, 10), 10);

		// When staking an amount below free balance.
		assert_eq!(staking_account.get_stakable_amount_for(&account, 5), 5);

		// When staking an amount above account free balance. It stakes all of the free balance.
		assert_eq!(staking_account.get_stakable_amount_for(&account, 25), 10);
	});
}

#[test]
fn impl_staking_target_details_increase_by() {
	new_test_ext().execute_with(|| {
		let mut staking_target = StakingTargetDetails::<BalanceOf<Test>>::default();
		assert_eq!(staking_target.increase_by(10, 10), Some(()));

		assert_eq!(
			staking_target,
			StakingTargetDetails::<BalanceOf<Test>> {
				amount: BalanceOf::<Test>::from(10u64),
				capacity: 10
			}
		)
	});
}

#[test]
fn impl_staking_capacity_details_increase_by() {
	new_test_ext().execute_with(|| {
		let mut capacity_details =
			CapacityDetails::<BalanceOf<Test>, <Test as Config>::EpochNumber>::default();
		assert_eq!(capacity_details.increase_by(10, 1), Some(()));

		assert_eq!(
			capacity_details,
			CapacityDetails::<BalanceOf<Test>, <Test as Config>::EpochNumber> {
				remaining: BalanceOf::<Test>::from(10u64),
				total_tokens_staked: BalanceOf::<Test>::from(10u64),
				total_available: BalanceOf::<Test>::from(10u64),
				last_replenished_epoch: <Test as Config>::EpochNumber::from(1u32)
			}
		)
	});
}

#[test]
fn staking_account_details_decrease_by_reduces_active_staking_balance_and_creates_unlock_chunk() {
	new_test_ext().execute_with(|| {
		let mut staking_account_details = StakingAccountDetails::<Test> {
			active: BalanceOf::<Test>::from(10u64),
			total: BalanceOf::<Test>::from(10u64),
			unlocking: BoundedVec::default(),
		};
		staking_account_details.decrease_by(3, 3).expect("decrease_by failed");
		let mut chunks: BoundedVec<
			UnlockChunk<BalanceOf<Test>, <Test as Config>::EpochNumber>,
			<Test as pallet_capacity::Config>::MaxUnlockingChunks,
		> = BoundedVec::default();

		chunks
			.try_push(UnlockChunk::<BalanceOf<Test>, <Test as Config>::EpochNumber> {
				value: BalanceOf::<Test>::from(3u64),
				thaw_at: <Test as Config>::EpochNumber::from(3u32),
			})
			.expect("try_push failed");

		assert_eq!(
			staking_account_details,
			StakingAccountDetails::<Test> {
				active: BalanceOf::<Test>::from(7u64),
				total: BalanceOf::<Test>::from(10u64),
				unlocking: chunks,
			}
		)
	});
}

#[test]
fn staking_target_details_decrease_by_reduces_staking_and_capacity_amounts() {
	new_test_ext().execute_with(|| {
		let mut staking_target_details = StakingTargetDetails::<BalanceOf<Test>> {
			amount: BalanceOf::<Test>::from(15u64),
			capacity: BalanceOf::<Test>::from(20u64),
		};
		staking_target_details.decrease_by(10, 10);

		assert_eq!(
			staking_target_details,
			StakingTargetDetails::<BalanceOf<Test>> {
				amount: BalanceOf::<Test>::from(5u64),
				capacity: BalanceOf::<Test>::from(10u64),
			}
		)
	});
}

#[test]
fn staking_capacity_details_decrease_by_reduces_total_tokens_staked_and_total_tokens_available() {
	new_test_ext().execute_with(|| {
		let mut capacity_details =
			CapacityDetails::<BalanceOf<Test>, <Test as Config>::EpochNumber> {
				remaining: BalanceOf::<Test>::from(10u64),
				total_tokens_staked: BalanceOf::<Test>::from(10u64),
				total_available: BalanceOf::<Test>::from(10u64),
				last_replenished_epoch: <Test as Config>::EpochNumber::from(0u32),
			};
		capacity_details.decrease_by(4, 5);

		assert_eq!(
			capacity_details,
			CapacityDetails::<BalanceOf<Test>, <Test as Config>::EpochNumber> {
				remaining: BalanceOf::<Test>::from(10u64),
				total_tokens_staked: BalanceOf::<Test>::from(5u64),
				total_available: BalanceOf::<Test>::from(6u64),
				last_replenished_epoch: <Test as Config>::EpochNumber::from(0u32)
			}
		)
	});
}
