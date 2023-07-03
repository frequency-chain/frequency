use super::mock::*;
use crate::*;
use common_primitives::node::{BlockNumber, RewardEra};
use frame_support::{assert_err, assert_ok, traits::Get};
use sp_core::bounded::BoundedVec;

type UnlockBVec<T> = BoundedVec<
	UnlockChunk<BalanceOf<T>, <T as Config>::EpochNumber>,
	<T as Config>::MaxUnlockingChunks,
>;

#[test]
fn staking_account_details_withdraw_reduces_active_staking_balance_and_creates_unlock_chunk() {
	let mut staking_account_details = StakingAccountDetails::<Test> {
		active: BalanceOf::<Test>::from(15u64),
		total: BalanceOf::<Test>::from(15u64),
		unlocking: BoundedVec::default(),
		staking_type: StakingType::MaximumCapacity,
		last_rewards_claimed_at: None,
		stake_change_unlocking: BoundedVec::default(),
	};
	assert_eq!(Ok(3u64), staking_account_details.withdraw(3, 3));
	let expected_chunks: UnlockBVec<Test> =
		BoundedVec::try_from(vec![UnlockChunk { value: 3u64, thaw_at: 3u32 }]).unwrap();

	assert_eq!(
		staking_account_details,
		StakingAccountDetails::<Test> {
			active: BalanceOf::<Test>::from(12u64),
			total: BalanceOf::<Test>::from(15u64),
			unlocking: expected_chunks,
			staking_type: StakingType::MaximumCapacity,
			last_rewards_claimed_at: None,
			stake_change_unlocking: BoundedVec::default(),
		}
	)
}

#[test]
fn staking_account_details_withdraw_goes_to_zero_when_result_below_minimum() {
	let mut staking_account_details = StakingAccountDetails::<Test> {
		active: BalanceOf::<Test>::from(10u64),
		total: BalanceOf::<Test>::from(10u64),
		unlocking: BoundedVec::default(),
		staking_type: StakingType::MaximumCapacity,
		last_rewards_claimed_at: None,
		stake_change_unlocking: BoundedVec::default(),
	};
	assert_eq!(Ok(10u64), staking_account_details.withdraw(6, 3));
	assert_eq!(0u64, staking_account_details.active);
	assert_eq!(10u64, staking_account_details.total);
}

#[test]
fn staking_account_details_withdraw_returns_err_when_too_many_chunks() {
	let maximum_chunks: UnlockBVec<Test> = BoundedVec::try_from(vec![
		UnlockChunk { value: 1u64, thaw_at: 3u32 },
		UnlockChunk { value: 1u64, thaw_at: 3u32 },
		UnlockChunk { value: 1u64, thaw_at: 3u32 },
		UnlockChunk { value: 1u64, thaw_at: 3u32 },
	])
	.unwrap();

	let mut staking_account_details = StakingAccountDetails::<Test> {
		active: BalanceOf::<Test>::from(10u64),
		total: BalanceOf::<Test>::from(10u64),
		unlocking: maximum_chunks,
		staking_type: StakingType::MaximumCapacity,
		last_rewards_claimed_at: None,
		stake_change_unlocking: BoundedVec::default(),
	};

	assert_err!(staking_account_details.withdraw(6, 3), Error::<Test>::MaxUnlockingChunksExceeded);
	assert_eq!(10u64, staking_account_details.active);
	assert_eq!(10u64, staking_account_details.total);
}

#[test]
fn staking_account_details_reap_thawed_happy_path() {
	let mut staking_account = StakingAccountDetails::<Test>::default();
	staking_account.deposit(10);

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
	let mut staking_account = StakingAccountDetails::<Test>::default();
	assert_eq!(staking_account.deposit(10), Some(()));

	assert_eq!(
		staking_account,
		StakingAccountDetails::<Test> {
			active: BalanceOf::<Test>::from(10u64),
			total: BalanceOf::<Test>::from(10u64),
			unlocking: BoundedVec::default(),
			staking_type: StakingType::MaximumCapacity,
			last_rewards_claimed_at: None,
			stake_change_unlocking: BoundedVec::default(),
		}
	)
}

#[test]
fn impl_staking_account_details_default() {
	assert_eq!(
		StakingAccountDetails::<Test>::default(),
		StakingAccountDetails::<Test> {
			active: BalanceOf::<Test>::zero(),
			total: BalanceOf::<Test>::zero(),
			unlocking: BoundedVec::default(),
			staking_type: StakingType::MaximumCapacity,
			last_rewards_claimed_at: None,
			stake_change_unlocking: BoundedVec::default(),
		},
	);
}

#[test]
fn impl_staking_account_details_get_stakable_amount_for() {
	new_test_ext().execute_with(|| {
		let account = 200;
		let staking_account = StakingAccountDetails::<Test>::default();

		// When staking all of free balance.
		assert_eq!(staking_account.get_stakable_amount_for(&account, 10), 10);

		// When staking an amount below free balance.
		assert_eq!(staking_account.get_stakable_amount_for(&account, 5), 5);

		// When staking an amount above account free balance. It stakes all of the free balance.
		assert_eq!(staking_account.get_stakable_amount_for(&account, 200), 190);
	});
}

#[test]
fn impl_update_stake_change_unlocking_bound() {
	new_test_ext().execute_with(|| {
		let mut staking_account: StakingAccountDetails<Test> = StakingAccountDetails {
			active: 150,
			total: 150,
			unlocking: Default::default(),
			staking_type: StakingType::MaximumCapacity,
			last_rewards_claimed_at: None,
			stake_change_unlocking: Default::default(),
		};
		let current_era_info: RewardEraInfo<RewardEra, BlockNumber> =
			RewardEraInfo { era_index: 20, started_at: 100 };
		let new_chunk_amount: u64 = 10;
		let thaw_at: u32 = 25;
		CurrentEraInfo::<Test>::set(current_era_info.clone());
		let max_chunks = <Test as Config>::MaxUnlockingChunks::get();
		for i in 0..max_chunks {
			assert_ok!(staking_account.update_stake_change_unlocking(
				&(new_chunk_amount + (i as u64)),
				&thaw_at,
				&current_era_info.era_index
			));
		}
		assert_err!(
			staking_account.update_stake_change_unlocking(
				&new_chunk_amount,
				&thaw_at,
				&current_era_info.era_index
			),
			Error::<Test>::MaxUnlockingChunksExceeded
		);
	})
}

#[test]
fn impl_update_stake_change_unlocking_cleanup() {
	new_test_ext().execute_with(|| {
		let mut staking_account: StakingAccountDetails<Test> = StakingAccountDetails {
			active: 150,
			total: 150,
			unlocking: Default::default(),
			staking_type: StakingType::MaximumCapacity,
			last_rewards_claimed_at: None,
			stake_change_unlocking: Default::default(),
		};
		let new_chunk_amount = 10u64;
		let thaw_at = 25u32;
		let era_index = 20u32;
		CurrentEraInfo::<Test>::set(RewardEraInfo { era_index, started_at: 100 });
		assert_ok!(staking_account.update_stake_change_unlocking(
			&new_chunk_amount,
			&thaw_at,
			&era_index
		));

		CurrentEraInfo::<Test>::set(RewardEraInfo { era_index: thaw_at, started_at: 100 });
		assert_ok!(staking_account.update_stake_change_unlocking(
			&new_chunk_amount,
			&thaw_at,
			&thaw_at
		));
		assert_eq!(1, staking_account.stake_change_unlocking.len());
	});
}
