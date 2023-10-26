use super::{
	mock::*,
	testing_utils::{setup_provider, staking_events},
};
use crate::{
	BalanceOf, CapacityDetails, Config, CurrentEraInfo, Error, Event, RetargetInfo, RewardEraInfo,
	StakingAccountDetails, StakingAccountLedger, StakingTargetDetails,
};
use common_primitives::{
	capacity::{
		StakingType,
		StakingType::{MaximumCapacity, ProviderBoost},
	},
	msa::MessageSourceId,
};
use frame_support::{assert_noop, assert_ok, traits::Get};

// staker is unused unless amount > 0
type TestCapacityDetails = CapacityDetails<BalanceOf<Test>, u32>;
type TestTargetDetails = StakingTargetDetails<BalanceOf<Test>>;

fn assert_capacity_details(
	msa_id: MessageSourceId,
	remaining_capacity: u64,
	total_tokens_staked: u64,
	total_capacity_issued: u64,
) {
	let expected_from_details: TestCapacityDetails = CapacityDetails {
		remaining_capacity,
		total_tokens_staked,
		total_capacity_issued,
		last_replenished_epoch: 0,
	};
	let from_capacity_details: TestCapacityDetails = Capacity::get_capacity_for(msa_id).unwrap();
	assert_eq!(from_capacity_details, expected_from_details);
}

fn assert_target_details(
	staker: u64,
	msa_id: MessageSourceId,
	amount: u64,
	capacity: u64,
	staking_type: StakingType,
) {
	let expected_from_target_details: TestTargetDetails =
		StakingTargetDetails { amount, capacity, staking_type };
	let from_target_details = Capacity::get_target_for(staker, msa_id).unwrap();
	assert_eq!(from_target_details, expected_from_target_details);
}

#[test]
fn do_retarget_happy_path() {
	new_test_ext().execute_with(|| {
		let staker = 10_000;
		let from_msa: MessageSourceId = 1;
		let from_amount = 600u64;
		let to_amount = 300u64;
		let to_msa: MessageSourceId = 2;
		let staking_type = ProviderBoost;
		setup_provider(&staker, &from_msa, &from_amount, staking_type.clone());
		setup_provider(&staker, &to_msa, &to_amount, staking_type.clone());

		// retarget half the stake to to_msa
		assert_ok!(Capacity::do_retarget(&staker, &from_msa, &to_msa, &to_amount, &staking_type));

		// expect from stake amounts to be halved
		assert_capacity_details(from_msa, 1, 300, 1);

		// expect to stake amounts to be increased by the retarget amount
		assert_capacity_details(to_msa, 3, 600, 3);

		assert_target_details(staker, from_msa, 300, 1, staking_type.clone());

		assert_target_details(staker, to_msa, 600, 3, staking_type.clone());
	})
}

#[test]
fn do_retarget_flip_flop() {
	new_test_ext().execute_with(|| {
		let staker = 10_000;
		let from_msa: MessageSourceId = 1;
		let from_amount = 600u64;
		let to_amount = 300u64;
		let to_msa: MessageSourceId = 2;
		setup_provider(&staker, &from_msa, &from_amount, ProviderBoost);
		setup_provider(&staker, &to_msa, &to_amount, ProviderBoost);

		for i in 0..4 {
			if i % 2 == 0 {
				assert_ok!(Capacity::do_retarget(
					&staker,
					&from_msa,
					&to_msa,
					&to_amount,
					&ProviderBoost
				));
			} else {
				assert_ok!(Capacity::do_retarget(
					&staker,
					&to_msa,
					&from_msa,
					&to_amount,
					&ProviderBoost
				));
			}
		}
		assert_capacity_details(from_msa, 3, 600, 3);
		assert_capacity_details(to_msa, 1, 300, 1);
	})
}

#[test]
// check that no capacity is minted or burned just by retargeting.
fn check_retarget_rounding_errors() {
	new_test_ext().execute_with(|| {
		let staker = 10_000;
		let from_msa: MessageSourceId = 1;
		let from_amount = 666u64;
		let to_amount = 301u64;
		let to_msa: MessageSourceId = 2;

		setup_provider(&staker, &from_msa, &from_amount, ProviderBoost);
		setup_provider(&staker, &to_msa, &to_amount, ProviderBoost);
		assert_capacity_details(from_msa, 3, 666, 3);
		assert_capacity_details(to_msa, 1, 301, 1);
		// 666+301= 967,  3+1=4

		assert_ok!(Capacity::do_retarget(&staker, &from_msa, &to_msa, &301u64, &ProviderBoost));
		assert_capacity_details(to_msa, 3, 602, 3);
		assert_capacity_details(from_msa, 1, 365, 1);
		// 602+365 = 967, 3+1 = 4

		assert_ok!(Capacity::do_retarget(&staker, &to_msa, &from_msa, &151u64, &ProviderBoost));
		assert_capacity_details(to_msa, 2, 451, 2);
		assert_capacity_details(from_msa, 2, 516, 2);
		// 451+516 = 967, 2+2 = 4
	})
}

fn assert_total_capacity(msas: Vec<MessageSourceId>, total: u64) {
	let sum = msas
		.into_iter()
		.map(|a| {
			let capacity_details: TestCapacityDetails = Capacity::get_capacity_for(a).unwrap();
			capacity_details.total_capacity_issued
		})
		.fold(0, |a, b| a + b);
	assert_eq!(total, sum);
}

#[test]
fn check_retarget_multiple_stakers() {
	new_test_ext().execute_with(|| {
		let staker_10k = 10_000;
		let staker_600 = 600u64;
		let staker_500 = 500u64;
		let staker_400 = 400u64;

		let from_msa: MessageSourceId = 1;
		let to_msa: MessageSourceId = 2;
		let amt1 = 192u64;
		let amt2 = 313u64;

		setup_provider(&staker_10k, &from_msa, &647u64, ProviderBoost);
		setup_provider(&staker_500, &to_msa, &293u64, ProviderBoost);
		assert_ok!(Capacity::stake(RuntimeOrigin::signed(staker_600.clone()), from_msa, 479u64,));
		assert_ok!(Capacity::stake(RuntimeOrigin::signed(staker_400.clone()), to_msa, 211u64,));

		// 647 * .1 * .05 = 3 (rounded down)
		// 293 * .1 * .05 = 1 (rounded down)
		// 479 * .1 = 48 (rounded up)
		// 211 * .1 = 21 (rounded down)
		// total capacity should be 73
		assert_total_capacity(vec![from_msa, to_msa], 73);

		assert_ok!(Capacity::do_retarget(&staker_10k, &from_msa, &to_msa, &amt2, &ProviderBoost));
		assert_ok!(Capacity::do_retarget(&staker_600, &from_msa, &to_msa, &amt1, &MaximumCapacity));
		assert_ok!(Capacity::do_retarget(&staker_500, &to_msa, &from_msa, &amt1, &ProviderBoost));
		assert_ok!(Capacity::do_retarget(&staker_400, &to_msa, &from_msa, &amt1, &MaximumCapacity));
		assert_total_capacity(vec![from_msa, to_msa], 73);
	})
}

#[test]
fn do_retarget_deletes_staking_target_details_if_zero_balance() {
	new_test_ext().execute_with(|| {
		let staker = 200u64;
		let from_msa: MessageSourceId = 1;
		let to_msa: MessageSourceId = 2;
		let amount = 10u64;
		setup_provider(&staker, &from_msa, &amount, MaximumCapacity);
		setup_provider(&staker, &to_msa, &amount, MaximumCapacity);

		// stake additional to provider from another Msa, doesn't matter which type.
		// total staked to from_msa is now 22u64.
		assert_ok!(Capacity::stake(RuntimeOrigin::signed(300u64), from_msa, 12u64,));

		assert_ok!(Capacity::do_retarget(&staker, &from_msa, &to_msa, &amount, &MaximumCapacity));

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

		let expected_to_target_details: TestTargetDetails = StakingTargetDetails {
			amount: 2 * amount,
			capacity: 2,
			staking_type: StakingType::MaximumCapacity,
		};
		let to_target_details = Capacity::get_target_for(staker, to_msa).unwrap();
		assert_eq!(to_target_details, expected_to_target_details);

		assert!(Capacity::get_target_for(staker, from_msa).is_none());
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
		setup_provider(&staker, &from_msa, &from_amount, ProviderBoost);
		setup_provider(&staker, &to_msa, &to_amount, ProviderBoost);

		assert_ok!(Capacity::change_staking_target(
			RuntimeOrigin::signed(staker),
			from_msa,
			to_msa,
			to_amount
		));
		let events = staking_events();

		assert_eq!(
			events.last().unwrap(),
			&Event::StakingTargetChanged { account: staker, from_msa, to_msa, amount: to_amount }
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
		setup_provider(&staker, &from_msa, &staking_amount, ProviderBoost);
		setup_provider(&staker, &to_msa, &10u64, ProviderBoost);

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
		setup_provider(&staking_account, &from_target, &staked_amount, ProviderBoost);
		setup_provider(&staking_account, &to_target, &staked_amount, ProviderBoost);

		CurrentEraInfo::<Test>::set(RewardEraInfo { era_index: 20, started_at: 100 });
		let max_chunks = <Test as Config>::MaxUnlockingChunks::get();
		for _i in 0..max_chunks {
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
			StakingAccountDetails { active: 20, total: 20, unlocking: Default::default() },
		);
		let from_account_not_staking = 100u64;
		let from_target_not_staked: MessageSourceId = 1;
		let to_target_not_provider: MessageSourceId = 2;
		let from_target: MessageSourceId = 3;
		let to_target: MessageSourceId = 4;
		setup_provider(&from_account, &from_target_not_staked, &0u64, ProviderBoost);
		setup_provider(&from_account, &from_target, &staked_amount, ProviderBoost);
		setup_provider(&from_account, &to_target, &staked_amount, ProviderBoost);

		assert_ok!(Capacity::provider_boost(
			RuntimeOrigin::signed(from_account),
			from_target,
			staked_amount,
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
				expected_err: Error::<Test>::StakerTargetRelationshipNotFound,
			},
			// from and to providers are valid, but zero amount too small
			TestCase {
				from_account,
				from_target,
				to_target,
				retarget_amount: 0,
				expected_err: Error::<Test>::StakingAmountBelowMinimum,
			},
			// nonzero amount below minimum is still too small
			TestCase {
				from_account,
				from_target,
				to_target,
				retarget_amount: 9,
				expected_err: Error::<Test>::StakingAmountBelowMinimum,
			},
			// account is staked with from-target, but to-target is not a provider
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
			TestCase {
				from_account,
				from_target,
				to_target: from_target,
				retarget_amount: 999,
				expected_err: Error::<Test>::CannotRetargetToSameProvider,
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

#[test]
fn impl_retarget_info_errors_when_attempt_to_update_past_bounded_max() {
	new_test_ext().execute_with(|| {
		struct TestCase {
			era: u32,
			retargets: u32,
			last_retarget: u32,
			expected: Option<()>,
		}
		for tc in [
			TestCase { era: 1u32, retargets: 0u32, last_retarget: 1, expected: Some(()) },
			TestCase { era: 1u32, retargets: 1u32, last_retarget: 1, expected: Some(()) },
			TestCase { era: 1u32, retargets: 1u32, last_retarget: 3, expected: Some(()) },
			TestCase { era: 1u32, retargets: 1u32, last_retarget: 4, expected: Some(()) },
			TestCase { era: 2u32, retargets: 5u32, last_retarget: 1, expected: Some(()) },
			TestCase { era: 1u32, retargets: 5u32, last_retarget: 1, expected: None },
		] {
			let mut retarget_info: RetargetInfo<Test> =
				RetargetInfo { retarget_count: tc.retargets, last_retarget_at: tc.last_retarget };
			assert_eq!(retarget_info.update(tc.era), tc.expected);
		}
	})
}

#[test]
fn impl_retarget_chunks_cleanup_when_new_reward_era() {
	new_test_ext().execute_with(|| {
		let current_era = 2u32;
		let mut retarget_info: RetargetInfo<Test> =
			RetargetInfo { retarget_count: 5, last_retarget_at: 1 };
		assert!(retarget_info.update(current_era).is_some());
		let expected: RetargetInfo<Test> = RetargetInfo { retarget_count: 1, last_retarget_at: 2 };
		assert_eq!(retarget_info, expected);
	});
}
