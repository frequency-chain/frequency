//! Unit tests for the time-release module.
use super::*;
use chrono::{DateTime, Days, Duration, TimeZone, Utc};
use frame_support::{
	assert_noop, assert_ok,
	error::BadOrigin,
	traits::{
		fungible::{Inspect, InspectFreeze, InspectHold},
		tokens::{Fortitude, WithdrawConsequence},
	},
};
use mock::*;
use pallet_scheduler::Agenda;
use sp_runtime::{traits::Dispatchable, SaturatedConversion, TokenError};

#[test]
fn time_release_from_chain_spec_works() {
	ExtBuilder::build().execute_with(|| {
		// Charlie has 30 tokens in total
		// 20 are frozen
		// 10 are free
		assert_eq!(PalletBalances::can_withdraw(&CHARLIE, 10), WithdrawConsequence::Success);

		// Charlie cannot withdraw more than 10 tokens
		assert_eq!(PalletBalances::can_withdraw(&CHARLIE, 11), WithdrawConsequence::Frozen);

		// Check that the release schedules built at genesis are correct
		assert_eq!(
			ReleaseSchedules::<Test>::get(&CHARLIE),
			vec![
				ReleaseSchedule { start: 2u32, period: 3u32, period_count: 1u32, per_period: 5u64 },
				ReleaseSchedule {
					start: 2u32 + 3u32,
					period: 3u32,
					period_count: 3u32,
					per_period: 5u64,
				}
			]
		);

		MockBlockNumberProvider::set(13);

		// 15 tokens will be released after 13 blocks
		// 25 = 10(free) + 15(released)
		assert_ok!(TimeRelease::claim(RuntimeOrigin::signed(CHARLIE)));
		assert_eq!(PalletBalances::can_withdraw(&CHARLIE, 25), WithdrawConsequence::Success);
		assert_eq!(PalletBalances::can_withdraw(&CHARLIE, 26), WithdrawConsequence::Frozen);

		MockBlockNumberProvider::set(14);

		// The final 5 tokens are released after 14 = (5[start] + 3[period] x 3[period count]) blocks
		assert_ok!(TimeRelease::claim(RuntimeOrigin::signed(CHARLIE)));

		// Charlie can now withdraw all 30 tokens
		// However, if Charlie doesn't leave an ED, the account will be reaped
		assert_eq!(
			PalletBalances::can_withdraw(&CHARLIE, 30),
			WithdrawConsequence::ReducedToZero(0)
		);
		assert_eq!(PalletBalances::can_withdraw(&CHARLIE, 29), WithdrawConsequence::Success);
	});
}

#[test]
fn transfer_works() {
	ExtBuilder::build().execute_with(|| {
		System::set_block_number(1);

		let schedule =
			ReleaseSchedule { start: 0u32, period: 10u32, period_count: 1u32, per_period: 100u64 };
		assert_ok!(TimeRelease::transfer(RuntimeOrigin::signed(ALICE), BOB, schedule.clone()));
		assert_eq!(ReleaseSchedules::<Test>::get(&BOB), vec![schedule.clone()]);
		System::assert_last_event(RuntimeEvent::TimeRelease(crate::Event::ReleaseScheduleAdded {
			from: ALICE,
			to: BOB,
			release_schedule: schedule,
		}));
	});
}

#[test]
fn self_releasing() {
	ExtBuilder::build().execute_with(|| {
		System::set_block_number(1);

		let schedule = ReleaseSchedule {
			start: 0u32,
			period: 10u32,
			period_count: 1u32,
			per_period: ALICE_BALANCE,
		};

		let bad_schedule = ReleaseSchedule {
			start: 0u32,
			period: 10u32,
			period_count: 1u32,
			per_period: 64 * ALICE_BALANCE,
		};

		assert_noop!(
			TimeRelease::transfer(RuntimeOrigin::signed(ALICE), ALICE, bad_schedule),
			crate::Error::<Test>::InsufficientBalanceToFreeze
		);

		assert_ok!(TimeRelease::transfer(RuntimeOrigin::signed(ALICE), ALICE, schedule.clone()));

		assert_eq!(ReleaseSchedules::<Test>::get(&ALICE), vec![schedule.clone()]);
		System::assert_last_event(RuntimeEvent::TimeRelease(crate::Event::ReleaseScheduleAdded {
			from: ALICE,
			to: ALICE,
			release_schedule: schedule,
		}));
	});
}

#[test]
fn add_new_release_schedule_merges_with_current_frozen_balance_and_until() {
	ExtBuilder::build().execute_with(|| {
		let schedule =
			ReleaseSchedule { start: 0u32, period: 10u32, period_count: 2u32, per_period: 10u64 };
		assert_ok!(TimeRelease::transfer(RuntimeOrigin::signed(ALICE), BOB, schedule));

		MockBlockNumberProvider::set(12);

		let another_schedule =
			ReleaseSchedule { start: 10u32, period: 13u32, period_count: 1u32, per_period: 7u64 };
		assert_ok!(TimeRelease::transfer(RuntimeOrigin::signed(ALICE), BOB, another_schedule));

		assert_eq!(
			<Test as Config>::Currency::balance_frozen(
				&FreezeReason::TimeReleaseVesting.into(),
				&BOB
			),
			17u64
		);
	});
}

#[test]
fn cannot_use_fund_if_not_claimed() {
	ExtBuilder::build().execute_with(|| {
		let schedule =
			ReleaseSchedule { start: 10u32, period: 10u32, period_count: 1u32, per_period: 50u64 };
		assert_ok!(TimeRelease::transfer(RuntimeOrigin::signed(ALICE), BOB, schedule));
		assert_eq!(PalletBalances::can_withdraw(&BOB, 1), WithdrawConsequence::Frozen);
	});
}

#[test]
fn transfer_fails_if_zero_period_or_count() {
	ExtBuilder::build().execute_with(|| {
		let schedule =
			ReleaseSchedule { start: 1u32, period: 0u32, period_count: 1u32, per_period: 100u64 };
		assert_noop!(
			TimeRelease::transfer(RuntimeOrigin::signed(ALICE), BOB, schedule),
			Error::<Test>::ZeroReleasePeriod
		);

		let schedule =
			ReleaseSchedule { start: 1u32, period: 1u32, period_count: 0u32, per_period: 100u64 };
		assert_noop!(
			TimeRelease::transfer(RuntimeOrigin::signed(ALICE), BOB, schedule),
			Error::<Test>::ZeroReleasePeriodCount
		);
	});
}

#[test]
fn transfer_fails_if_transfer_err() {
	ExtBuilder::build().execute_with(|| {
		let schedule =
			ReleaseSchedule { start: 1u32, period: 1u32, period_count: 1u32, per_period: 100u64 };
		assert_noop!(
			TimeRelease::transfer(RuntimeOrigin::signed(BOB), ALICE, schedule),
			TokenError::FundsUnavailable
		);
	});
}

#[test]
fn transfer_fails_if_overflow() {
	ExtBuilder::build().execute_with(|| {
		let schedule =
			ReleaseSchedule { start: 1u32, period: 1u32, period_count: 2u32, per_period: u64::MAX };
		assert_noop!(
			TimeRelease::transfer(RuntimeOrigin::signed(ALICE), BOB, schedule),
			ArithmeticError::Overflow,
		);

		let another_schedule =
			ReleaseSchedule { start: u32::MAX, period: 1u32, period_count: 2u32, per_period: 1u64 };
		assert_noop!(
			TimeRelease::transfer(RuntimeOrigin::signed(ALICE), BOB, another_schedule),
			ArithmeticError::Overflow,
		);
	});
}

#[test]
fn transfer_fails_if_bad_origin() {
	ExtBuilder::build().execute_with(|| {
		let schedule =
			ReleaseSchedule { start: 0u32, period: 10u32, period_count: 1u32, per_period: 100u64 };
		assert_noop!(
			TimeRelease::transfer(RuntimeOrigin::signed(CHARLIE), BOB, schedule),
			BadOrigin
		);
	});
}

#[test]
fn claim_works() {
	ExtBuilder::build().execute_with(|| {
		let schedule =
			ReleaseSchedule { start: 0u32, period: 10u32, period_count: 2u32, per_period: 10u64 };
		assert_ok!(TimeRelease::transfer(RuntimeOrigin::signed(ALICE), BOB, schedule));

		MockBlockNumberProvider::set(11);
		// remain frozen if not claimed
		assert!(
			PalletBalances::transfer_allow_death(RuntimeOrigin::signed(BOB), ALICE, 10).is_err()
		);
		// thawed after claiming
		assert_ok!(TimeRelease::claim(RuntimeOrigin::signed(BOB)));
		assert!(ReleaseSchedules::<Test>::contains_key(BOB));
		assert_ok!(PalletBalances::transfer_allow_death(RuntimeOrigin::signed(BOB), ALICE, 10));
		// more are still frozen
		assert!(PalletBalances::transfer_allow_death(RuntimeOrigin::signed(BOB), ALICE, 1).is_err());

		MockBlockNumberProvider::set(21);
		// claim more
		assert_ok!(TimeRelease::claim(RuntimeOrigin::signed(BOB)));
		assert!(!ReleaseSchedules::<Test>::contains_key(BOB));
		assert_ok!(PalletBalances::transfer_allow_death(RuntimeOrigin::signed(BOB), ALICE, 10));
		// all used up
		assert_eq!(<Test as Config>::Currency::balance(&BOB), 0);

		// none frozen anymore
		assert_eq!(
			<Test as Config>::Currency::balance_frozen(
				&FreezeReason::TimeReleaseVesting.into(),
				&BOB
			),
			0
		);
	});
}

#[test]
fn claim_for_works() {
	ExtBuilder::build().execute_with(|| {
		let schedule =
			ReleaseSchedule { start: 0u32, period: 10u32, period_count: 2u32, per_period: 10u64 };
		assert_ok!(TimeRelease::transfer(RuntimeOrigin::signed(ALICE), BOB, schedule));

		assert_ok!(TimeRelease::claim_for(RuntimeOrigin::signed(ALICE), BOB));

		assert_eq!(
			<Test as Config>::Currency::balance_frozen(
				&FreezeReason::TimeReleaseVesting.into(),
				&BOB
			),
			20u64
		);
		assert!(ReleaseSchedules::<Test>::contains_key(&BOB));

		MockBlockNumberProvider::set(21);

		assert_ok!(TimeRelease::claim_for(RuntimeOrigin::signed(ALICE), BOB));

		// none frozen anymore
		assert_eq!(
			<Test as Config>::Currency::balance_frozen(
				&FreezeReason::TimeReleaseVesting.into(),
				&BOB
			),
			0
		);
		assert!(!ReleaseSchedules::<Test>::contains_key(&BOB));
	});
}

#[test]
fn update_release_schedules_works() {
	ExtBuilder::build().execute_with(|| {
		let schedule =
			ReleaseSchedule { start: 0u32, period: 10u32, period_count: 2u32, per_period: 10u64 };
		assert_ok!(TimeRelease::transfer(RuntimeOrigin::signed(ALICE), BOB, schedule));

		let updated_schedule =
			ReleaseSchedule { start: 0u32, period: 20u32, period_count: 2u32, per_period: 10u64 };
		assert_ok!(TimeRelease::update_release_schedules(
			RuntimeOrigin::root(),
			BOB,
			vec![updated_schedule]
		));

		MockBlockNumberProvider::set(11);
		assert_ok!(TimeRelease::claim(RuntimeOrigin::signed(BOB)));
		assert!(PalletBalances::transfer_allow_death(RuntimeOrigin::signed(BOB), ALICE, 1).is_err());

		MockBlockNumberProvider::set(21);
		assert_ok!(TimeRelease::claim(RuntimeOrigin::signed(BOB)));
		assert_ok!(PalletBalances::transfer_allow_death(RuntimeOrigin::signed(BOB), ALICE, 10));

		// empty release schedules cleanup the storage and thaw the funds
		assert!(ReleaseSchedules::<Test>::contains_key(BOB));
		assert_eq!(
			<Test as Config>::Currency::balance_frozen(
				&FreezeReason::TimeReleaseVesting.into(),
				&BOB
			),
			10u64
		);
		assert_ok!(TimeRelease::update_release_schedules(RuntimeOrigin::root(), BOB, vec![]));
		assert!(!ReleaseSchedules::<Test>::contains_key(BOB));
		assert_eq!(
			<Test as Config>::Currency::balance_frozen(
				&FreezeReason::TimeReleaseVesting.into(),
				&BOB
			),
			0
		);
	});
}

#[test]
fn update_release_schedules_fails_if_unexpected_existing_freezes() {
	ExtBuilder::build().execute_with(|| {
		assert_ok!(PalletBalances::transfer_allow_death(RuntimeOrigin::signed(ALICE), BOB, 1));
		let _ = <Test as Config>::Currency::set_freeze(
			&FreezeReason::TimeReleaseVesting.into(),
			&BOB,
			0u64,
		);
	});
}

#[test]
fn transfer_check_for_min() {
	ExtBuilder::build().execute_with(|| {
		let schedule =
			ReleaseSchedule { start: 1u32, period: 1u32, period_count: 1u32, per_period: 3u64 };
		assert_noop!(
			TimeRelease::transfer(RuntimeOrigin::signed(BOB), ALICE, schedule),
			TokenError::FundsUnavailable
		);
	});
}

#[test]
fn multiple_release_schedule_claim_works() {
	ExtBuilder::build().execute_with(|| {
		let schedule =
			ReleaseSchedule { start: 0u32, period: 10u32, period_count: 2u32, per_period: 10u64 };
		assert_ok!(TimeRelease::transfer(RuntimeOrigin::signed(ALICE), BOB, schedule.clone()));

		let schedule2 =
			ReleaseSchedule { start: 0u32, period: 10u32, period_count: 3u32, per_period: 10u64 };
		assert_ok!(TimeRelease::transfer(RuntimeOrigin::signed(ALICE), BOB, schedule2.clone()));

		assert_eq!(ReleaseSchedules::<Test>::get(&BOB), vec![schedule, schedule2.clone()]);

		MockBlockNumberProvider::set(21);

		assert_ok!(TimeRelease::claim(RuntimeOrigin::signed(BOB)));

		assert_eq!(ReleaseSchedules::<Test>::get(&BOB), vec![schedule2]);

		MockBlockNumberProvider::set(31);

		assert_ok!(TimeRelease::claim(RuntimeOrigin::signed(BOB)));

		assert!(!ReleaseSchedules::<Test>::contains_key(&BOB));

		assert_eq!(
			<Test as Config>::Currency::balance_frozen(
				&FreezeReason::TimeReleaseVesting.into(),
				&BOB
			),
			0
		);
	});
}

#[test]
fn exceeding_maximum_schedules_should_fail() {
	ExtBuilder::build().execute_with(|| {
		set_balance::<Test>(&ALICE, 1000);

		let schedule =
			ReleaseSchedule { start: 0u32, period: 10u32, period_count: 2u32, per_period: 10u64 };

		for _ in 0..49 {
			assert_ok!(TimeRelease::transfer(RuntimeOrigin::signed(ALICE), BOB, schedule.clone()));
		}

		assert_ok!(TimeRelease::transfer(RuntimeOrigin::signed(ALICE), BOB, schedule.clone()));

		let create = RuntimeCall::TimeRelease(crate::Call::<Test>::transfer {
			dest: BOB,
			schedule: schedule.clone(),
		});

		assert_noop!(
			create.dispatch(RuntimeOrigin::signed(ALICE)),
			Error::<Test>::MaxReleaseSchedulesExceeded
		);

		let schedules = vec![schedule.clone(); 51];

		assert_noop!(
			TimeRelease::update_release_schedules(RuntimeOrigin::root(), BOB, schedules),
			Error::<Test>::MaxReleaseSchedulesExceeded
		);
	});
}

#[test]
fn schedule_transfer_success() {
	ExtBuilder::build().execute_with(|| {
		let current_relay_block_number = 15u32;
		let current_frequency_block_number = 5u32;

		run_to_block(current_frequency_block_number);
		MockBlockNumberProvider::set(current_relay_block_number);

		let schedule = ReleaseSchedule {
			// relay block number
			start: 25u32.into(),
			period: 5u32.into(),
			period_count: 2u32,
			per_period: 10u32.into(),
		};

		let schedule_frequency_blocknumber =
			BlockNumberFor::<Test>::from(current_relay_block_number + 5);

		assert_ok!(TimeRelease::schedule_named_transfer(
			RuntimeOrigin::signed(ALICE),
			[1u8; 32],
			BOB,
			schedule,
			schedule_frequency_blocknumber,
		));

		let schedules = Agenda::<Test>::get(schedule_frequency_blocknumber);
		assert_eq!(schedules.len(), 1);

		let hold_balance = PalletBalances::balance_on_hold(
			&HoldReason::TimeReleaseScheduledVesting.into(),
			&ALICE,
		);
		assert_eq!(hold_balance, 20u64);

		run_to_block(current_relay_block_number + 5);
		let schedules = Agenda::<Test>::get(4u32);
		assert_eq!(schedules.len(), 0);

		let hold_balance_at_execution = PalletBalances::balance_on_hold(
			&HoldReason::TimeReleaseScheduledVesting.into(),
			&ALICE,
		);
		assert_eq!(hold_balance_at_execution, 0u64);

		assert_eq!(PalletBalances::free_balance(BOB), 20u64);
		assert_eq!(
			PalletBalances::balance_frozen(&FreezeReason::TimeReleaseVesting.into(), &BOB),
			20u64
		);
	});
}

#[test]
fn multiple_schedule_transfer_with_same_schedule_name_fails() {
	ExtBuilder::build().execute_with(|| {
		let schedule = ReleaseSchedule {
			start: 6u32.into(),
			period: 10u32.into(),
			period_count: 2u32,
			per_period: 10u32.into(),
		};

		let when = BlockNumberFor::<Test>::from(4u32);

		assert_ok!(TimeRelease::schedule_named_transfer(
			RuntimeOrigin::signed(ALICE),
			[1u8; 32],
			BOB,
			schedule,
			when
		));
		let schedules = Agenda::<Test>::get(4u32);
		assert_eq!(schedules.len(), 1);
		let hold_balance = PalletBalances::balance_on_hold(
			&HoldReason::TimeReleaseScheduledVesting.into(),
			&ALICE,
		);
		assert_eq!(hold_balance, 20u64);

		// Adding another schedule increases the hold

		let schedule_2 = ReleaseSchedule {
			start: 6u32.into(),
			period: 10u32.into(),
			period_count: 2u32,
			per_period: 10u32.into(),
		};

		let when = BlockNumberFor::<Test>::from(4u32);

		assert_noop!(
			TimeRelease::schedule_named_transfer(
				RuntimeOrigin::signed(ALICE),
				[1u8; 32],
				BOB,
				schedule_2,
				when
			),
			Error::<Test>::DuplicateScheduleName
		);
	});
}

#[test]
fn multiple_schedule_transfer_success() {
	ExtBuilder::build().execute_with(|| {
		let schedule = ReleaseSchedule {
			start: 6u32.into(),
			period: 10u32.into(),
			period_count: 2u32,
			per_period: 10u32.into(),
		};

		let when = BlockNumberFor::<Test>::from(4u32);

		assert_ok!(TimeRelease::schedule_named_transfer(
			RuntimeOrigin::signed(ALICE),
			[1u8; 32],
			BOB,
			schedule,
			when
		));
		let schedules = Agenda::<Test>::get(4u32);
		assert_eq!(schedules.len(), 1);
		let hold_balance = PalletBalances::balance_on_hold(
			&HoldReason::TimeReleaseScheduledVesting.into(),
			&ALICE,
		);
		assert_eq!(hold_balance, 20u64);

		// Adding another schedule increases the hold

		let schedule_2 = ReleaseSchedule {
			start: 6u32.into(),
			period: 10u32.into(),
			period_count: 2u32,
			per_period: 10u32.into(),
		};

		let when = BlockNumberFor::<Test>::from(4u32);

		assert_ok!(TimeRelease::schedule_named_transfer(
			RuntimeOrigin::signed(ALICE),
			[2u8; 32],
			BOB,
			schedule_2,
			when
		));
		let schedules = Agenda::<Test>::get(4u32);
		assert_eq!(schedules.len(), 2);
		let hold_balance = PalletBalances::balance_on_hold(
			&HoldReason::TimeReleaseScheduledVesting.into(),
			&ALICE,
		);
		assert_eq!(hold_balance, 40u64);
	});
}

#[test]
fn schedule_transfer_fails_if_invalid_block_number() {
	ExtBuilder::build().execute_with(|| {
		let schedule = ReleaseSchedule {
			start: 6u32.into(),
			period: 10u32.into(),
			period_count: 2u32,
			per_period: 10u32.into(),
		};
		let invalid_block_number = 0u32;
		assert_noop!(
			TimeRelease::schedule_named_transfer(
				RuntimeOrigin::signed(ALICE),
				[1u8; 32],
				BOB,
				schedule,
				invalid_block_number
			),
			pallet_scheduler::Error::<Test>::TargetBlockNumberInPast
		);
	});
}

#[test]
fn schedule_transfer_fails_if_bad_origin() {
	ExtBuilder::build().execute_with(|| {
		let schedule = ReleaseSchedule {
			start: 6u32.into(),
			period: 10u32.into(),
			period_count: 2u32,
			per_period: 10u32.into(),
		};
		let when = BlockNumberFor::<Test>::from(4u32);
		assert_noop!(
			TimeRelease::schedule_named_transfer(
				RuntimeOrigin::signed(CHARLIE),
				[1u8; 32],
				BOB,
				schedule,
				when
			),
			BadOrigin
		);
	});
}

#[test]
fn execute_scheduled_named_transfer_success() {
	ExtBuilder::build().execute_with(|| {
		let schedule = ReleaseSchedule {
			start: 6u32.into(),
			period: 10u32.into(),
			period_count: 2u32,
			per_period: 10u32.into(),
		};
		let when = BlockNumberFor::<Test>::from(4u32);
		assert_ok!(TimeRelease::schedule_named_transfer(
			RuntimeOrigin::signed(ALICE),
			[1u8; 32],
			BOB,
			schedule.clone(),
			when
		));

		let amount = ScheduleReservedAmounts::<Test>::get([1u8; 32]);
		assert_eq!(20u64, amount.unwrap());

		assert_ok!(TimeRelease::execute_scheduled_named_transfer(
			Origin::<Test>::TimeRelease(ALICE).into(),
			[1u8; 32],
			BOB,
			schedule
		));

		assert_eq!(PalletBalances::free_balance(BOB), 20u64);
		assert_eq!(None, ScheduleReservedAmounts::<Test>::get([1u8; 32]));
	});
}

#[test]
fn cancel_schedule_named_transfer_success() {
	ExtBuilder::build().execute_with(|| {
		run_to_block(2u32);

		let schedule = ReleaseSchedule {
			start: 6u32.into(),
			period: 10u32.into(),
			period_count: 2u32,
			per_period: 10u32.into(),
		};
		let when = BlockNumberFor::<Test>::from(4u32);
		assert_ok!(TimeRelease::schedule_named_transfer(
			RuntimeOrigin::signed(ALICE),
			[1u8; 32],
			BOB,
			schedule.clone(),
			when
		));

		let agendas = Agenda::<Test>::get(4u32);
		assert_eq!(agendas.len(), 1);
		assert_eq!(ScheduleReservedAmounts::<Test>::get([1u8; 32]), Some(20u64));

		assert_ok!(TimeRelease::cancel_scheduled_named_transfer(
			RuntimeOrigin::signed(ALICE),
			[1u8; 32]
		));

		assert_eq!(ScheduleReservedAmounts::<Test>::get([1u8; 32]), None);
		assert_eq!(Agenda::<Test>::get(4u32).len(), 0);
	});
}
#[test]
fn cancel_schedule_transfer_errors_not_found() {
	ExtBuilder::build().execute_with(|| {
		assert_noop!(
			TimeRelease::cancel_scheduled_named_transfer(RuntimeOrigin::signed(BOB), [1u8; 32]),
			pallet_scheduler::Error::<Test>::NotFound
		);
	});
}

#[test]
fn cancel_schedule_transfer_bad_origin_fails() {
	ExtBuilder::build().execute_with(|| {
		run_to_block(2u32);

		let schedule = ReleaseSchedule {
			start: 6u32.into(),
			period: 10u32.into(),
			period_count: 2u32,
			per_period: 10u32.into(),
		};
		let when = BlockNumberFor::<Test>::from(4u32);
		assert_ok!(TimeRelease::schedule_named_transfer(
			RuntimeOrigin::signed(ALICE),
			[1u8; 32],
			BOB,
			schedule.clone(),
			when
		));

		let agendas = Agenda::<Test>::get(4u32);
		assert_eq!(agendas.len(), 1);

		assert_noop!(
			TimeRelease::cancel_scheduled_named_transfer(RuntimeOrigin::signed(BOB), [1u8; 32]),
			BadOrigin
		);
	});
}

#[test]
fn execute_scheduled_named_transfer_fails_if_insufficient_hold_balance() {
	ExtBuilder::build().execute_with(|| {
		let schedule = ReleaseSchedule {
			start: 6u32.into(),
			period: 10u32.into(),
			period_count: 2u32,
			per_period: 10u32.into(),
		};
		let when = BlockNumberFor::<Test>::from(4u32);
		assert_ok!(TimeRelease::schedule_named_transfer(
			RuntimeOrigin::signed(ALICE),
			[1u8; 32],
			BOB,
			schedule.clone(),
			when
		));

		let bad_schedule = ReleaseSchedule {
			start: 6u32.into(),
			period: 10u32.into(),
			period_count: 2u32,
			per_period: 1000u32.into(),
		};

		assert_noop!(
			TimeRelease::execute_scheduled_named_transfer(
				Origin::<Test>::TimeRelease(ALICE).into(),
				[1u8; 32],
				BOB,
				bad_schedule
			),
			Error::<Test>::InsufficientBalanceToFreeze
		);
	});
}

#[test]
fn execute_scheduled_named_transfer_fails_if_bad_origin() {
	ExtBuilder::build().execute_with(|| {
		let schedule = ReleaseSchedule {
			start: 6u32.into(),
			period: 10u32.into(),
			period_count: 2u32,
			per_period: 10u32.into(),
		};
		let when = BlockNumberFor::<Test>::from(4u32);
		assert_ok!(TimeRelease::schedule_named_transfer(
			RuntimeOrigin::signed(ALICE),
			[1u8; 32],
			BOB,
			schedule.clone(),
			when
		));

		assert_noop!(
			TimeRelease::execute_scheduled_named_transfer(
				RuntimeOrigin::signed(CHARLIE),
				[1u8; 32],
				BOB,
				schedule
			),
			BadOrigin
		);
	});
}

#[test]
fn cliff_release_works() {
	const VESTING_AMOUNT: u64 = 12;
	const VESTING_PERIOD: u32 = 20;

	ExtBuilder::build().execute_with(|| {
		let cliff_schedule = ReleaseSchedule {
			start: VESTING_PERIOD - 1,
			period: 1,
			period_count: 1,
			per_period: VESTING_AMOUNT,
		};

		assert_eq!(<Test as Config>::Currency::balance(&BOB), 0);
		assert_ok!(TimeRelease::transfer(RuntimeOrigin::signed(ALICE), BOB, cliff_schedule));
		assert_eq!(<Test as Config>::Currency::balance(&BOB), VESTING_AMOUNT);
		assert_eq!(
			<Test as Config>::Currency::balance_frozen(
				&FreezeReason::TimeReleaseVesting.into(),
				&BOB
			),
			VESTING_AMOUNT
		);

		for i in 1..VESTING_PERIOD {
			MockBlockNumberProvider::set(i);
			assert_ok!(TimeRelease::claim(RuntimeOrigin::signed(BOB)));
			assert_eq!(<Test as Config>::Currency::balance(&BOB), VESTING_AMOUNT);
			assert_eq!(
				<Test as Config>::Currency::balance_frozen(
					&FreezeReason::TimeReleaseVesting.into(),
					&BOB
				),
				VESTING_AMOUNT
			);
			assert_noop!(
				PalletBalances::transfer_allow_death(
					RuntimeOrigin::signed(BOB),
					CHARLIE,
					VESTING_AMOUNT
				),
				TokenError::Frozen,
			);
		}

		MockBlockNumberProvider::set(VESTING_PERIOD);
		assert_ok!(TimeRelease::claim(RuntimeOrigin::signed(BOB)));
		assert_eq!(
			<Test as Config>::Currency::balance_frozen(
				&FreezeReason::TimeReleaseVesting.into(),
				&BOB
			),
			0
		);
		assert_ok!(PalletBalances::transfer_allow_death(
			RuntimeOrigin::signed(BOB),
			CHARLIE,
			VESTING_AMOUNT
		));
	});
}

/// Example:
/// Alice has 100k tokens
/// Alice gets 25k tokens on April 1
/// Note: 25k = 1/4th = 6/24ths of total
/// Alice's 25k tokens will thaw:
/// - 1/24th of total (1/6th of the 25k) on July 1st 2024
/// - 2/24th of total (2/6th of the 25k) on Aug 1st 2024 (+432_000 blocks)
/// - 3/24th of total (3/6th of the 25k) on Sep 1st 2024
/// - 4/24th of total (4/6th of the 25k) on Oct 1st 2024
/// - 5/24th of total (5/6th of the 25k) on Nov 1st 2024
/// - 6/24th of total (6/6th of the 25k) on Dec 1st 2024
/// - 7/24th of total (2/3rd of quarterly) on Jan 1st 2025 (Q1 Award)
/// - 15/48th of total (1/3rd of quarterly) on Feb 1st 2025 (Q1 Award)
/// - 8/24th of total (1/3rd of quarterly) on Feb 1st 2025 (Q2 Award)
/// - 9/24th of total (2/3rd of quarterly) on Mar 1st 2025 (Q2 Award)
#[test]
fn alice_time_releases_schedule() {
	ExtBuilder::build().execute_with(|| {
		// First Thaw = ~ July 1, 2024
		// Thaw monthly
		// "Start" freeze June 1, 2024
		let total_award: u64 = 100_000;
		let amount_released_per_period = total_award / 24;
		let number_of_periods = 6u32;
		let period_duration = Duration::days(30);

		let july_2024_first_release_date = Utc.with_ymd_and_hms(2024, 7, 1, 0, 0, 0).unwrap();
		let june_2024_release_start = july_2024_first_release_date - Days::new(30); // block_number: 21_041_037

		let schedule = build_time_release_schedule::<Test>(
			june_2024_release_start,
			period_duration,
			number_of_periods,
			amount_released_per_period,
		);

		set_balance::<Test>(&ALICE, total_award);

		// Bob starts with zero balance and zero frozen.
		assert_eq!(get_balance::<Test>(&BOB), 0);
		assert_eq!(
			<Test as Config>::Currency::balance_frozen(
				&FreezeReason::TimeReleaseVesting.into(),
				&BOB
			),
			0
		);

		// Time release transfer is initiated by Alice to Bob. As a result, Bobs free-balance
		// increases by the total amount scheduled to be time-released.
		// However, it cannot spent because a freeze is put on the balance.
		assert_ok!(TimeRelease::transfer(RuntimeOrigin::signed(ALICE), BOB, schedule));
		assert_eq!(get_balance::<Test>(&BOB), 24_996);
		assert_eq!(
			<Test as Config>::Currency::balance_frozen(
				&FreezeReason::TimeReleaseVesting.into(),
				&BOB
			),
			24996
		);
		assert_eq!(
			<Test as Config>::Currency::reducible_balance(
				&BOB,
				Preservation::Expendable,
				Fortitude::Polite
			),
			0
		);

		// Bob naively attempts to claim the transfer before the scheduled release date
		// and nothing happens because the schedule release has yet to start.
		assert_ok!(TimeRelease::claim(RuntimeOrigin::signed(BOB)));
		assert_eq!(
			<Test as Config>::Currency::balance_frozen(
				&FreezeReason::TimeReleaseVesting.into(),
				&BOB
			),
			24_996
		);

		let time_release_transfer_data: Vec<(DateTime<Utc>, _)> =
			time_release_transfers_data::<Test>();

		// quarters 1 - 5
		let july_1_2023_to_july_1_2024_data = &time_release_transfer_data[0..4];

		// time travel and create transfer for each date. These transfers increase the total amount frozen in Bobs account.
		let mut total_frozen = amount_released_per_period * 6;
		for transfer in july_1_2023_to_july_1_2024_data.iter() {
			let transfer_1 = transfer.1.get(0).unwrap().clone();
			let transfer_2 = transfer.1.get(1).unwrap().clone();
			total_frozen += transfer_1.per_period + transfer_2.per_period;

			assert_ok!(TimeRelease::transfer(RuntimeOrigin::signed(ALICE), BOB, transfer_1,));
			assert_ok!(TimeRelease::transfer(RuntimeOrigin::signed(ALICE), BOB, transfer_2));
		}

		// Bob thinks he can claim tokens because time-release transfer has started.
		// However, Bob notices he still can't spend it. That is because he has not
		// gone through 1 period =(.
		MockBlockNumberProvider::set(
			date_to_approximate_block_number(june_2024_release_start).into(),
		);
		// Doing a claim does not do anything
		assert_ok!(TimeRelease::claim(RuntimeOrigin::signed(BOB)));
		// Since the first issuance the total amount frozen increases by the new transfers: 24_996;
		assert_eq!(
			<Test as Config>::Currency::balance_frozen(
				&FreezeReason::TimeReleaseVesting.into(),
				&BOB
			),
			total_frozen
		);

		let july_2024_sept_2024: Vec<DateTime<Utc>> = vec![
			Utc.with_ymd_and_hms(2024, 7, 1, 0, 0, 0).unwrap(),
			Utc.with_ymd_and_hms(2024, 8, 1, 0, 0, 0).unwrap(),
			Utc.with_ymd_and_hms(2024, 9, 1, 0, 0, 0).unwrap(),
		];

		for month in july_2024_sept_2024.iter() {
			// Bob trys again at the end of the first period. Bob is now
			// happy because he has thawed 4_166 tokens every month and can spend them.
			// The total amount is reduced by the amount released per period.
			MockBlockNumberProvider::set(date_to_approximate_block_number(month.clone()).into());
			assert_ok!(TimeRelease::claim(RuntimeOrigin::signed(BOB)));
			total_frozen -= 4_166 as u64;
			assert_eq!(
				<Test as Config>::Currency::balance_frozen(
					&FreezeReason::TimeReleaseVesting.into(),
					&BOB
				),
				total_frozen
			);
		}

		// quarter-6:  time-release transfer AND monthly claim
		// Note that when submitting a transfer it also makes a claim and releases transfers.
		let oct_2024_quarterly_data = &time_release_transfer_data[5];
		MockBlockNumberProvider::set(
			date_to_approximate_block_number(oct_2024_quarterly_data.0).into(),
		);

		let transfer_1 = oct_2024_quarterly_data.1.get(0).unwrap();
		assert_ok!(TimeRelease::transfer(RuntimeOrigin::signed(ALICE), BOB, transfer_1.clone(),));

		let transfer_2 = oct_2024_quarterly_data.1.get(1).unwrap();
		assert_ok!(TimeRelease::transfer(RuntimeOrigin::signed(ALICE), BOB, transfer_2.clone(),));
		let total_transfered =
			transfer_1.total_amount().unwrap() + transfer_2.total_amount().unwrap();

		// new transfer_total - one_month_of_time_release
		total_frozen += total_transfered;
		total_frozen -= 4_166;
		assert_eq!(
			<Test as Config>::Currency::balance_frozen(
				&FreezeReason::TimeReleaseVesting.into(),
				&BOB
			),
			total_frozen
		);

		// quarter-7-12:  time-release transfer AND monthly claim
		let jan_2025_to_april_2026_quarterly_data = &time_release_transfer_data[6..];
		for quarter in jan_2025_to_april_2026_quarterly_data.iter() {
			MockBlockNumberProvider::set(date_to_approximate_block_number(quarter.0).into());

			let transfer_1 = quarter.1.get(0).unwrap();
			assert_ok!(TimeRelease::transfer(
				RuntimeOrigin::signed(ALICE),
				BOB,
				transfer_1.clone(),
			));

			let transfer_2 = quarter.1.get(1).unwrap();
			assert_ok!(TimeRelease::transfer(
				RuntimeOrigin::signed(ALICE),
				BOB,
				transfer_2.clone(),
			));
		}

		MockBlockNumberProvider::set(
			date_to_approximate_block_number(Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 6).unwrap())
				.into(),
		);
		assert_ok!(TimeRelease::claim(RuntimeOrigin::signed(BOB)));
		assert_eq!(
			<Test as Config>::Currency::balance_frozen(
				&FreezeReason::TimeReleaseVesting.into(),
				&BOB
			),
			0
		);
	});
}

fn get_balance<T: Config>(who: &T::AccountId) -> BalanceOf<T> {
	T::Currency::balance(who)
}

fn set_balance<T: Config>(who: &T::AccountId, balance: BalanceOf<T>) {
	let actual_deposit = T::Currency::mint_into(&who, balance.saturated_into());
	assert_eq!(balance, actual_deposit.unwrap());
}

fn build_time_release_schedule<T: Config>(
	start_date: DateTime<Utc>,
	period: Duration,
	period_count: u32,
	per_period: BalanceOf<T>,
) -> ReleaseSchedule<BlockNumberFor<T>, BalanceOf<T>> {
	let start_block_number = date_to_approximate_block_number(start_date);

	let days_in_seconds = period.num_seconds();
	let number_of_blocks_in_days = days_in_seconds / 6;

	ReleaseSchedule::<BlockNumberFor<T>, BalanceOf<T>> {
		start: start_block_number.into(),
		period: (number_of_blocks_in_days as u32).into(),
		period_count,
		per_period: per_period.into(),
	}
}

fn date_to_approximate_block_number(input_date: DateTime<Utc>) -> u32 {
	let average_block_time = 6;

	let initial_block_time = Utc.with_ymd_and_hms(2023, 3, 24, 22, 42, 0).unwrap();
	let block_number_at_initial_block_time: i64 = 14_790_657;

	let duration = input_date.signed_duration_since(initial_block_time);
	let seconds_elapsed = duration.num_seconds();

	let block_number = (seconds_elapsed / average_block_time) + block_number_at_initial_block_time;

	block_number as u32
}

// Bob also receives additional transfers quarterl transfers for
// (6,250 tokens trasfer quarterly for 12 quarters for a total of 75,000 totals vested)
// Quarter 1: July 2023
// Quarter 2: Oct 2023
// Quarter 3: Jan 2024
// Quarter 4 : April 2024
// Quarter 5 : July 2024
// Quarter 6: Oct 2024
// Quarter 7: Jan 2025
// Quarter 8: April 2025
// Quarter 9: July 2025
// Quarter 10: Oct 2025
// Quarter 11: Jan 2026
// Quarter 12: April 2026
// Time-Relese schedules:
// Monthly thaws 1 / 24 of 100K starting July 1, 2024
fn time_release_transfers_data<T: Config>() -> Vec<(DateTime<Utc>, Vec<ReleaseSchedule<u32, u64>>)>
{
	let total_award: u64 = 100_000;
	vec![
		(
			// Quarter 1: July 2023: Time-release transfer = 6_250            Release schedule of 1/24 of 100K per month
			Utc.with_ymd_and_hms(2023, 7, 1, 0, 0, 0).unwrap(),
			vec![
				// 7/24
				build_time_release_schedule::<Test>(
					Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
					Duration::seconds(6),
					1u32,
					(total_award * 1 / 24).into(), // Max amount available for release: 100k / 24 ~ 4_166
				),
				// 8/24
				build_time_release_schedule::<Test>(
					Utc.with_ymd_and_hms(2025, 2, 1, 0, 0, 0).unwrap(),
					Duration::seconds(6),
					1,
					total_award * 1 / 24 * 1 / 2, // Remaining vested: 6_250 - 4_166 ~ 2_084/
				), // Note 2_084 is short of the total amount that could be released from 4_166.
			],
		),
		(
			// Quarter 2: Oct 2023: Time-release transfer = 6_250
			Utc.with_ymd_and_hms(2023, 10, 1, 0, 0, 0).unwrap(),
			vec![
				// 8/24
				build_time_release_schedule::<Test>(
					// 6_250 is issue and can be partially released on 2025-2-1 to complete the amount eligible for release.
					Utc.with_ymd_and_hms(2025, 2, 1, 0, 0, 0).unwrap(),
					Duration::seconds(6),
					1,
					total_award * 1 / 24 * 1 / 2, // total_reward is 6_250 - 4_166 ~ 2_084
				),
				// 9/24
				build_time_release_schedule::<Test>(
					Utc.with_ymd_and_hms(2025, 3, 1, 0, 0, 0).unwrap(),
					Duration::seconds(6),
					1,
					total_award * 1 / 24, // ~ 4_166
				),
			],
		),
		(
			// Quarter 3: Jan 2024: Time-release transfer = 6_250
			Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(), // Max amount available for release: 100k / 24 ~ 4_166
			vec![
				// 10/24
				build_time_release_schedule::<Test>(
					Utc.with_ymd_and_hms(2025, 4, 1, 0, 0, 0).unwrap(),
					Duration::seconds(6),
					1,
					total_award * 1 / 24,
				),
				// 11/24
				build_time_release_schedule::<Test>(
					Utc.with_ymd_and_hms(2025, 5, 1, 0, 0, 0).unwrap(),
					Duration::seconds(6),
					1,                            // Remaining vested: 6_250 - 4_166 ~ 2_084/
					total_award * 1 / 24 * 1 / 2, // Note 2_084 is short of the total amount that could be released from 4_166.
				),
			],
		),
		(
			// Quarter 4 : April 2024
			Utc.with_ymd_and_hms(2024, 4, 1, 0, 0, 0).unwrap(),
			vec![
				// 11/24
				build_time_release_schedule::<Test>(
					// previous bucket
					Utc.with_ymd_and_hms(2025, 5, 1, 0, 0, 0).unwrap(),
					Duration::seconds(6),
					1,
					total_award * 1 / 24 * 1 / 2,
				),
				// 12/24
				build_time_release_schedule::<Test>(
					Utc.with_ymd_and_hms(2025, 6, 1, 0, 0, 0).unwrap(),
					Duration::seconds(6),
					1,
					total_award * 1 / 24,
				),
			],
		),
		(
			// Quarter 5: Jul 2024
			Utc.with_ymd_and_hms(2024, 7, 1, 0, 0, 0).unwrap(),
			vec![
				// 13/24
				build_time_release_schedule::<Test>(
					Utc.with_ymd_and_hms(2025, 7, 1, 0, 0, 0).unwrap(),
					Duration::seconds(6),
					1,
					total_award * 1 / 24,
				),
				// 14/24
				build_time_release_schedule::<Test>(
					Utc.with_ymd_and_hms(2025, 8, 1, 0, 0, 0).unwrap(),
					Duration::seconds(6),
					1,
					total_award * 1 / 24 * 1 / 2,
				),
			],
		),
		(
			// Quarter 6: Oct 2024
			Utc.with_ymd_and_hms(2024, 10, 1, 0, 0, 0).unwrap(),
			vec![
				// 14/24
				build_time_release_schedule::<Test>(
					Utc.with_ymd_and_hms(2025, 8, 1, 0, 0, 0).unwrap(),
					Duration::seconds(6),
					1,
					total_award * 1 / 24 * 1 / 2,
				),
				// 15/24
				build_time_release_schedule::<Test>(
					Utc.with_ymd_and_hms(2025, 9, 1, 0, 0, 0).unwrap(),
					Duration::seconds(6),
					1,
					total_award * 1 / 24,
				),
			],
		),
		(
			// Quarter 7: Jan 2025
			Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
			vec![
				// 16/24
				build_time_release_schedule::<Test>(
					Utc.with_ymd_and_hms(2025, 10, 1, 0, 0, 0).unwrap(),
					Duration::seconds(6),
					1,
					total_award * 1 / 24,
				),
				// 17/24
				build_time_release_schedule::<Test>(
					Utc.with_ymd_and_hms(2025, 11, 1, 0, 0, 0).unwrap(),
					Duration::seconds(6),
					1,
					total_award * 1 / 24 * 1 / 2,
				),
			],
		),
		(
			// Quarter 8: April 2025
			Utc.with_ymd_and_hms(2025, 4, 1, 0, 0, 0).unwrap(), // issued     6_250
			vec![
				// 17/24
				build_time_release_schedule::<Test>(
					Utc.with_ymd_and_hms(2025, 11, 1, 0, 0, 0).unwrap(),
					Duration::seconds(6),
					1,
					total_award * 1 / 24 * 1 / 2, // 6_250 - 4_166 ~ 2_084
				),
				// 18/24
				build_time_release_schedule::<Test>(
					Utc.with_ymd_and_hms(2025, 12, 1, 0, 0, 0).unwrap(),
					Duration::seconds(6),
					1,
					total_award * 1 / 24, // ~ 100K / 24 ~ 4_166
				),
			],
		),
		(
			// Quarter 9: July 2025
			Utc.with_ymd_and_hms(2025, 7, 1, 0, 0, 0).unwrap(), // issued     6_250
			vec![
				// 19/24
				build_time_release_schedule::<Test>(
					Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
					Duration::seconds(6),
					1,
					total_award * 1 / 24, // ~ 100K / 24 ~ 4_166
				),
				// 20/24
				build_time_release_schedule::<Test>(
					Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).unwrap(),
					Duration::seconds(6),
					1,
					total_award * 1 / 24 * 1 / 2, // 6_250 - 4_166 ~ 2_084
				),
			],
		),
		(
			// Quarter 10: Oct 2025
			Utc.with_ymd_and_hms(2025, 10, 1, 0, 0, 0).unwrap(), // issued     6_250
			vec![
				// 20/24
				build_time_release_schedule::<Test>(
					Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).unwrap(),
					Duration::seconds(6),
					1,
					total_award * 1 / 24 * 1 / 2, // 6_250 - 4_166 ~ 2_084
				),
				// 21/24
				build_time_release_schedule::<Test>(
					Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0).unwrap(),
					Duration::seconds(6),
					1,
					total_award * 1 / 24, // ~ 100K / 24 ~ 4_166
				),
			],
		),
		(
			// Quarter 11: Jan 2026
			Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(), // issued     6_250
			vec![
				// 22/24
				build_time_release_schedule::<Test>(
					Utc.with_ymd_and_hms(2026, 4, 1, 0, 0, 0).unwrap(),
					Duration::seconds(6),
					1,
					total_award * 1 / 24, // ~ 100K / 24 ~ 4_166
				),
				// 23/24
				build_time_release_schedule::<Test>(
					Utc.with_ymd_and_hms(2026, 5, 1, 0, 0, 0).unwrap(),
					Duration::seconds(6),
					1,
					total_award * 1 / 24 * 1 / 2, // 6_250 - 4_166 ~ 2_084
				),
			],
		),
		(
			// Quarter 12: April 2026
			Utc.with_ymd_and_hms(2026, 4, 1, 0, 0, 0).unwrap(),
			vec![
				// 23/24
				build_time_release_schedule::<Test>(
					Utc.with_ymd_and_hms(2026, 5, 1, 0, 0, 0).unwrap(),
					Duration::seconds(6),
					1,
					total_award * 1 / 24 * 1 / 2, // 6_250 - 4_166 ~ 2_084
				),
				// 24/24
				build_time_release_schedule::<Test>(
					Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap(),
					Duration::seconds(6),
					1,
					total_award * 1 / 24, // ~ 100K / 24 ~ 4_166
				),
			],
		),
	]
}
