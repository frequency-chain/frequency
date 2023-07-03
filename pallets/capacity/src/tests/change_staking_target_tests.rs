use super::{mock::*, testing_utils::*};
use crate::{
	BalanceOf, CapacityDetails, Config, CurrentEraInfo, Error, Event, RewardEraInfo,
	StakingAccountDetails, StakingAccountLedger, StakingTargetDetails,
};
use common_primitives::{
	capacity::StakingType::{MaximumCapacity, ProviderBoost},
	msa::MessageSourceId,
};
use frame_support::{assert_noop, assert_ok, traits::Get};

// staker is unused unless amount > 0
fn setup_provider(staker: u64, target: MessageSourceId, amount: u64) {
	let provider_name = String::from("Cst-") + target.to_string().as_str();
	register_provider(target, provider_name);
	if amount > 0 {
		assert_ok!(Capacity::stake(RuntimeOrigin::signed(staker), target, amount, ProviderBoost));
	}
}

type TestCapacityDetails = CapacityDetails<BalanceOf<Test>, u32>;
type TestTargetDetails = StakingTargetDetails<BalanceOf<Test>>;

#[test]
fn set_capacity_for_deletes_staking_target_details_if_amount_is_zero() {
	new_test_ext().execute_with(|| {
		let token_account = 200;
		let target: MessageSourceId = 1;
		let target2: MessageSourceId = 2;
		// minimum amount + 1 so it doesn't zero out the balance when we unstake
		let staking_amount = 11;

		register_provider(target, String::from("Test Target"));
		register_provider(target2, String::from("Test Target2"));

		// stake to both of them so we still have a staking balance overall
		assert_ok!(Capacity::stake(
			RuntimeOrigin::signed(token_account),
			target,
			staking_amount,
			MaximumCapacity
		));
		assert_ok!(Capacity::stake(
			RuntimeOrigin::signed(token_account),
			target2,
			staking_amount,
			MaximumCapacity
		));
		assert_ok!(Capacity::unstake(RuntimeOrigin::signed(token_account), target, staking_amount));
		assert_noop!(
			Capacity::unstake(RuntimeOrigin::signed(token_account), target, 1),
			Error::<Test>::StakerTargetRelationshipNotFound
		);
	})
}

#[test]
fn do_retarget_happy_path() {
	new_test_ext().execute_with(|| {
		let staker = 200u64;
		let from_msa: MessageSourceId = 1;
		let from_amount = 20u64;
		let to_amount = from_amount / 2;
		let to_msa: MessageSourceId = 2;
		setup_provider(staker, from_msa, from_amount);
		setup_provider(staker, to_msa, to_amount);

		// retarget half the stake to to_msa
		assert_ok!(Capacity::do_retarget(&staker, &from_msa, &to_msa, &to_amount));

		// expect from stake amounts to be halved
		let expected_from_details: TestCapacityDetails = CapacityDetails {
			remaining_capacity: 1,
			total_tokens_staked: 10,
			total_capacity_issued: 1,
			last_replenished_epoch: 0,
		};
		let from_capacity_details: TestCapacityDetails =
			Capacity::get_capacity_for(from_msa).unwrap();
		assert_eq!(from_capacity_details, expected_from_details);

		// expect to stake amounts to be increased by the retarget amount
		let expected_to_details: TestCapacityDetails = CapacityDetails {
			remaining_capacity: 2,
			total_tokens_staked: 20,
			total_capacity_issued: 2,
			last_replenished_epoch: 0,
		};
		let to_capacity_details = Capacity::get_capacity_for(to_msa).unwrap();
		assert_eq!(to_capacity_details, expected_to_details);

		let expected_from_target_details: TestTargetDetails =
			StakingTargetDetails { amount: 10, capacity: 1 };
		let from_target_details = Capacity::get_target_for(staker, from_msa).unwrap();
		assert_eq!(from_target_details, expected_from_target_details);

		let expected_to_target_details: TestTargetDetails =
			StakingTargetDetails { amount: 20, capacity: 2 };
		let to_target_details = Capacity::get_target_for(staker, to_msa).unwrap();
		assert_eq!(to_target_details, expected_to_target_details);
	})
}

#[test]
fn do_retarget_deletes_staking_account_if_zero_balance() {
	new_test_ext().execute_with(|| {
		let staker = 200u64;
		let from_msa: MessageSourceId = 1;
		let to_msa: MessageSourceId = 2;
		let amount = 10u64;
		setup_provider(staker, from_msa, amount);
		setup_provider(staker, to_msa, amount);

		// stake additional to provider from another Msa, doesn't matter which type.
		// total staked to from_msa is now 22u64.
		assert_ok!(Capacity::stake(
			RuntimeOrigin::signed(300u64),
			from_msa,
			12u64,
			MaximumCapacity
		));

		assert_ok!(Capacity::do_retarget(&staker, &from_msa, &to_msa, &amount));

		let expected_from_details: TestCapacityDetails = CapacityDetails {
			remaining_capacity: 1,
			total_tokens_staked: 12,
			total_capacity_issued: 1,
			last_replenished_epoch: 0,
		};
		let from_capacity_details: TestCapacityDetails =
			Capacity::get_capacity_for(from_msa).unwrap();
		assert_eq!(from_capacity_details, expected_from_details);

		let expected_to_details: TestCapacityDetails = CapacityDetails {
			remaining_capacity: 2,
			total_tokens_staked: 2 * amount,
			total_capacity_issued: 2,
			last_replenished_epoch: 0,
		};

		let to_capacity_details = Capacity::get_capacity_for(to_msa).unwrap();
		assert_eq!(to_capacity_details, expected_to_details);

		assert!(Capacity::get_target_for(staker, from_msa).is_none());

		let expected_to_target_details: TestTargetDetails =
			StakingTargetDetails { amount: 2 * amount, capacity: 2 };
		let to_target_details = Capacity::get_target_for(staker, to_msa).unwrap();
		assert_eq!(to_target_details, expected_to_target_details);
	})
}

#[test]
fn change_staking_starget_emits_event_on_success() {
	new_test_ext().execute_with(|| {
		let staker = 200u64;
		let from_msa: MessageSourceId = 1;
		let from_amount = 20u64;
		let to_amount = from_amount / 2;
		let to_msa: MessageSourceId = 2;
		setup_provider(staker, from_msa, from_amount);
		setup_provider(staker, to_msa, to_amount);

		assert_ok!(Capacity::change_staking_target(
			RuntimeOrigin::signed(staker),
			from_msa,
			to_msa,
			to_amount
		));
		let events = staking_events();

		assert_eq!(
			events.last().unwrap(),
			&Event::CapacityTargetChanged { account: staker, from_msa, to_msa, amount: to_amount }
		);
	})
}

#[test]
fn change_staking_target_errors_if_too_many_changes_before_thaw() {
	new_test_ext().execute_with(|| {
		let staker = 200u64;
		let from_msa: MessageSourceId = 1;
		let to_msa: MessageSourceId = 2;

		let max_chunks: u32 = <Test as Config>::MaxUnlockingChunks::get();
		let staking_amount = ((max_chunks + 2u32) * 10u32) as u64;
		setup_provider(staker, from_msa, staking_amount);
		setup_provider(staker, to_msa, 10);

		let retarget_amount = 10u64;
		for _i in 0..(max_chunks) {
			assert_ok!(Capacity::change_staking_target(
				RuntimeOrigin::signed(staker),
				from_msa,
				to_msa,
				retarget_amount
			));
		}

		assert_noop!(
			Capacity::change_staking_target(
				RuntimeOrigin::signed(staker),
				from_msa,
				to_msa,
				retarget_amount
			),
			Error::<Test>::MaxUnlockingChunksExceeded
		);
	});
}

#[test]
fn change_staking_target_garbage_collects_thawed_chunks() {
	new_test_ext().execute_with(|| {
		let staked_amount = 50u64;
		let staking_account = 200u64;
		let from_target: MessageSourceId = 3;
		let to_target: MessageSourceId = 4;
		setup_provider(staking_account, from_target, staked_amount);
		setup_provider(staking_account, to_target, staked_amount);

		CurrentEraInfo::<Test>::set(RewardEraInfo { era_index: 20, started_at: 100 });
		let max_chunks = <Test as Config>::MaxUnlockingChunks::get();
		for i in 0..max_chunks {
			println!("{:?}", i);
			assert_ok!(Capacity::change_staking_target(
				RuntimeOrigin::signed(staking_account),
				from_target,
				to_target,
				10u64,
			));
		}
		CurrentEraInfo::<Test>::set(RewardEraInfo { era_index: 25, started_at: 100 });
		assert_ok!(Capacity::change_staking_target(
			RuntimeOrigin::signed(staking_account),
			from_target,
			to_target,
			10u64,
		));
	})
}

#[test]
fn change_staking_target_test_parametric_validity() {
	new_test_ext().execute_with(|| {
		let staked_amount = 10u64;
		let from_account = 200u64;

		StakingAccountLedger::<Test>::insert(
			from_account,
			StakingAccountDetails {
				active: 20,
				total: 20,
				unlocking: Default::default(),
				staking_type: ProviderBoost,
				last_rewards_claimed_at: None,
				stake_change_unlocking: Default::default(),
			},
		);
		let from_account_not_staking = 100u64;
		let from_target_not_staked: MessageSourceId = 1;
		let to_target_not_provider: MessageSourceId = 2;
		let from_target: MessageSourceId = 3;
		let to_target: MessageSourceId = 4;
		setup_provider(from_account, from_target_not_staked, 0);
		setup_provider(from_account, from_target, staked_amount);
		setup_provider(from_account, to_target, staked_amount);

		assert_ok!(Capacity::stake(
			RuntimeOrigin::signed(from_account),
			from_target,
			staked_amount,
			ProviderBoost
		));

		struct TestCase {
			from_account: u64,
			from_target: MessageSourceId,
			to_target: MessageSourceId,
			retarget_amount: u64,
			expected_err: Error<Test>,
		}
		let test_cases: Vec<TestCase> = vec![
			// from is a provider but account is not staking to it
			TestCase {
				from_account,
				from_target: from_target_not_staked,
				to_target,
				retarget_amount: staked_amount,
				expected_err: Error::<Test>::StakerTargetRelationshipNotFound,
			},
			// from_account is not staking at all.
			TestCase {
				from_account: from_account_not_staking,
				from_target,
				to_target,
				retarget_amount: staked_amount,
				expected_err: Error::<Test>::StakingAccountNotFound,
			},
			// // from and to providers are valid, but zero amount too small
			TestCase {
				from_account,
				from_target,
				to_target,
				retarget_amount: 0,
				expected_err: Error::<Test>::StakingAmountBelowMinimum,
			},
			// // nonzero amount below minimum is still too small
			TestCase {
				from_account,
				from_target,
				to_target,
				retarget_amount: 9,
				expected_err: Error::<Test>::StakingAmountBelowMinimum,
			},
			// // account is staked with from-target, but to-target is not a provider
			TestCase {
				from_account,
				from_target,
				to_target: to_target_not_provider,
				retarget_amount: staked_amount,
				expected_err: Error::<Test>::InvalidTarget,
			},
			// account doesn't have enough staked to make the transfer
			TestCase {
				from_account,
				from_target,
				to_target,
				retarget_amount: 999,
				expected_err: Error::<Test>::InsufficientStakingBalance,
			},
		];

		for tc in test_cases {
			assert_noop!(
				Capacity::change_staking_target(
					RuntimeOrigin::signed(tc.from_account),
					tc.from_target,
					tc.to_target,
					tc.retarget_amount,
				),
				tc.expected_err
			);
		}
	});
}
