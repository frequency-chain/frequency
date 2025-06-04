use crate::{
	tests::{
		mock::{
			assert_transferable, get_balance, new_test_ext, Capacity, RuntimeOrigin, System, Test,
		},
		testing_utils::{run_to_block, setup_provider},
	},
	CurrentEraInfo, Error, Event,
	Event::ProviderBoostRewardClaimed,
	MessageSourceId, ProviderBoostHistories,
	StakingType::*,
};
use frame_support::{assert_noop, assert_ok};

#[test]
fn claim_staking_rewards_leaves_one_history_item_for_current_era() {
	new_test_ext().execute_with(|| {
		let account = 10_000u64;
		let target: MessageSourceId = 10;
		let amount = 1_000u64;

		setup_provider(&account, &target, &amount, CommittedBoost);
		run_to_block(31);
		assert_eq!(CurrentEraInfo::<Test>::get().era_index, 3u32);

		let current_history = ProviderBoostHistories::<Test>::get(account).unwrap();
		assert_eq!(current_history.count(), 1usize);
		let history_item = current_history.get_entry_for_era(&0u32).unwrap();
		assert_eq!(*history_item, amount);
	})
}

#[test]
fn claim_staking_rewards_allows_unstake() {
	new_test_ext().execute_with(|| {
		let account = 10_000u64;
		let target: MessageSourceId = 10;
		let amount = 1_000u64;

		setup_provider(&account, &target, &amount, CommittedBoost);
		run_to_block(31);
		assert_noop!(
			Capacity::unstake(RuntimeOrigin::signed(account), target, 100u64),
			Error::<Test>::MustFirstClaimRewards
		);
		assert_ok!(Capacity::claim_staking_rewards(RuntimeOrigin::signed(account)));
		assert_ok!(Capacity::unstake(RuntimeOrigin::signed(account), target, 400u64));
	})
}

#[test]
fn claim_staking_rewards_mints_and_transfers_expected_total() {
	new_test_ext().execute_with(|| {
		let account = 10_000u64;
		let target: MessageSourceId = 10;
		let amount = 1_000u64;

		setup_provider(&account, &target, &amount, CommittedBoost);
		run_to_block(31);
		assert_eq!(CurrentEraInfo::<Test>::get().era_index, 3u32);
		assert_ok!(Capacity::claim_staking_rewards(RuntimeOrigin::signed(account)));
		System::assert_last_event(
			Event::<Test>::ProviderBoostRewardClaimed { account, reward_amount: 8u64 }.into(),
		);

		// should have 2 era's worth of payouts: 4 each for eras 1, 2
		assert_eq!(get_balance::<Test>(&account), 10_008u64);

		// the reward value is unlocked
		assert_transferable::<Test>(&account, 8u64);

		run_to_block(41);
		assert_eq!(CurrentEraInfo::<Test>::get().era_index, 4u32);
		assert_ok!(Capacity::claim_staking_rewards(RuntimeOrigin::signed(account)));
		// rewards available for one more era
		System::assert_last_event(
			ProviderBoostRewardClaimed { account, reward_amount: 4u64 }.into(),
		);
		// should have 4 for eras 1-3
		assert_eq!(get_balance::<Test>(&account), 10_012u64);
		assert_transferable::<Test>(&account, 12u64);
	})
}

#[test]
fn claim_staking_rewards_has_expected_total_when_other_stakers() {
	new_test_ext().execute_with(|| {
		let account = 10_000u64;
		let target: MessageSourceId = 10;
		let amount = 1_000u64;

		let other_staker = 600_u64;
		let other_amount = 300_u64;
		setup_provider(&other_staker, &target, &other_amount, CommittedBoost);
		assert_ok!(Capacity::provider_boost(RuntimeOrigin::signed(account), target, amount));
		run_to_block(31); // 4
		assert_ok!(Capacity::claim_staking_rewards(RuntimeOrigin::signed(account)));
		assert_eq!(get_balance::<Test>(&account), 10_008u64);
	})
}

#[test]
fn claim_staking_rewards_has_expected_total_if_amount_should_be_capped() {
	new_test_ext().execute_with(|| {
		let account = 10_000u64;
		let target: MessageSourceId = 10;
		let amount = 9_900u64;
		setup_provider(&account, &target, &amount, CommittedBoost);
		run_to_block(31);
		assert_ok!(Capacity::claim_staking_rewards(RuntimeOrigin::signed(account)));
		assert_eq!(get_balance::<Test>(&account), 10_076u64); // cap is 0.38%,*2 = 76, rounded
		assert_transferable::<Test>(&account, 76u64);
	})
}

#[test]
fn claim_staking_rewards_fails_if_no_available_rewards() {
	new_test_ext().execute_with(|| {
		let account = 10_000u64;
		let target: MessageSourceId = 10;
		let amount = 1_000u64;

		setup_provider(&account, &target, &amount, CommittedBoost);
		// Unclaimed rewards should have zero length b/c it's still Era 1.
		// Nothing will be in history
		assert_noop!(
			Capacity::claim_staking_rewards(RuntimeOrigin::signed(account)),
			Error::<Test>::NoRewardsEligibleToClaim
		);

		run_to_block(15); // Era is 2, but still not available for staking rewards until era 3.
		assert_noop!(
			Capacity::claim_staking_rewards(RuntimeOrigin::signed(account)),
			Error::<Test>::NoRewardsEligibleToClaim
		);
	})
}

#[test]
fn claim_staking_rewards_fails_if_stake_maximized() {
	new_test_ext().execute_with(|| {
		let account = 10_000u64;
		let target: MessageSourceId = 10;
		let amount = 1_000u64;

		assert_noop!(
			Capacity::claim_staking_rewards(RuntimeOrigin::signed(account)),
			Error::<Test>::NotAProviderBoostAccount
		);

		setup_provider(&account, &target, &amount, MaximumCapacity);

		assert_noop!(
			Capacity::claim_staking_rewards(RuntimeOrigin::signed(account)),
			Error::<Test>::NotAProviderBoostAccount
		);
	})
}
