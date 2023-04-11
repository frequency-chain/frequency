use crate::{
	tests::{
		mock::{new_test_ext, Capacity, RuntimeOrigin, Test},
		testing_utils::{run_to_block, setup_provider, system_run_to_block},
	},
	Config, ProviderBoostHistory,
	StakingType::{MaximumCapacity, ProviderBoost},
};
use common_primitives::capacity::RewardEra;
use frame_support::assert_ok;
use sp_runtime::traits::{Get, Zero};

#[test]
fn provider_boost_adds_to_staking_history() {
	new_test_ext().execute_with(|| {
		let staker = 10_000u64;
		let target = 1u64;

		setup_provider(&staker, &target, &1_000u64, ProviderBoost);
		let history = Capacity::get_staking_history_for(staker);
		assert!(history.is_some());
	})
}

#[test]
fn multiple_provider_boosts_updates_history_correctly() {
	new_test_ext().execute_with(|| {
		let staker = 10_000u64;
		let target = 1u64;
		setup_provider(&staker, &target, &500u64, ProviderBoost);

		assert_ok!(Capacity::provider_boost(RuntimeOrigin::signed(staker), target, 200));

		// should update era 1 history
		let mut history = Capacity::get_staking_history_for(staker).unwrap();
		assert_eq!(history.count(), 1);
		assert_eq!(history.get_entry_for_era(&1u32).unwrap(), &700u64);

		system_run_to_block(10);
		run_to_block(11);

		assert_ok!(Capacity::provider_boost(RuntimeOrigin::signed(staker), target, 200));

		// should add an era 2 history
		history = Capacity::get_staking_history_for(staker).unwrap();
		assert_eq!(history.count(), 2);
		assert_eq!(history.get_entry_for_era(&2u32).unwrap(), &900u64);
	})
}

#[test]
fn stake_does_not_add_to_staking_history() {
	new_test_ext().execute_with(|| {
		let staker = 10_000u64;
		let target = 1u64;

		setup_provider(&staker, &target, &1_000u64, MaximumCapacity);
		let history = Capacity::get_staking_history_for(staker);
		assert!(history.is_none());
	})
}

#[test]
fn provider_boost_history_add_era_balance_adds_entries_and_deletes_old_if_full() {
	let mut pbh = ProviderBoostHistory::<Test>::new();
	let bound: u32 = <Test as Config>::ProviderBoostHistoryLimit::get();

	for era in 1..bound + 1u32 {
		let add_amount: u64 = 1_000 * era as u64;
		pbh.add_era_balance(&era, &add_amount);
		assert_eq!(pbh.count(), era as usize);
	}
	assert_eq!(pbh.count(), bound as usize);

	let new_era: RewardEra = 99;
	pbh.add_era_balance(&new_era, &1000u64);
	assert_eq!(pbh.count(), bound as usize);
	assert!(pbh.get_entry_for_era(&new_era).is_some());

	let first_era: RewardEra = 1;
	assert!(pbh.get_entry_for_era(&first_era).is_none());
}

#[test]
fn provider_boost_history_add_era_balance_in_same_era_updates_entry() {
	let mut pbh = ProviderBoostHistory::<Test>::new();
	let add_amount: u64 = 1_000u64;
	let era = 2u32;
	pbh.add_era_balance(&era, &add_amount);
	pbh.add_era_balance(&era, &add_amount);
	assert_eq!(pbh.get_entry_for_era(&era).unwrap(), &2_000u64)
}

#[test]
fn provider_boost_history_subtract_era_balance_in_same_era_updates_entry() {
	let mut pbh = ProviderBoostHistory::<Test>::new();
	let era = 2u32;
	pbh.add_era_balance(&era, &1000u64);
	pbh.subtract_era_balance(&era, &300u64);
	assert_eq!(pbh.get_entry_for_era(&(era)).unwrap(), &700u64);
}

#[test]
fn provider_boost_history_add_era_balance_in_new_era_correctly_adds_value() {
	let mut pbh = ProviderBoostHistory::<Test>::new();
	let add_amount: u64 = 1_000u64;
	let era = 2u32;
	pbh.add_era_balance(&era, &add_amount);
	pbh.add_era_balance(&(era + 1), &add_amount);
	assert_eq!(pbh.get_entry_for_era(&(era + 1)).unwrap(), &2_000u64)
}

#[test]
fn provider_boost_history_subtract_era_balance_in_new_era_correctly_subtracts_value() {
	let mut pbh = ProviderBoostHistory::<Test>::new();
	let era = 2u32;
	pbh.add_era_balance(&era, &1000u64);

	pbh.subtract_era_balance(&(era + 1), &400u64);
	assert_eq!(pbh.get_entry_for_era(&(era + 1)).unwrap(), &600u64);

	pbh.subtract_era_balance(&(era + 2), &600u64);
	assert!(pbh.get_entry_for_era(&(era + 2)).unwrap().is_zero());
}

#[test]
fn provider_boost_history_subtract_all_balance_on_only_entry_returns_some_0() {
	let mut pbh = ProviderBoostHistory::<Test>::new();
	let era = 22u32;
	let amount = 1000u64;
	pbh.add_era_balance(&era, &amount);
	assert_eq!(pbh.subtract_era_balance(&(era), &amount), Some(0usize));
}
