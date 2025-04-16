#![allow(clippy::unwrap_used)]
use super::*;

use crate::Pallet as TimeReleasePallet;

use frame_benchmarking::{account, benchmarks, whitelist_account, whitelisted_caller};
use frame_support::traits::fungible::InspectHold;
use frame_system::{pallet_prelude::BlockNumberFor, RawOrigin};
use sp_runtime::{traits::TrailingZeroInput, SaturatedConversion};
extern crate alloc;
use alloc::vec;

pub const DOLLARS: u32 = 10u32.saturating_pow(8u32);

pub use crate::types::ReleaseSchedule;
pub type Schedule<T> = ReleaseSchedule<BlockNumberFor<T>, BalanceOf<T>>;

const SEED: u32 = 0;

fn set_balance<T: Config>(who: &T::AccountId, balance: BalanceOf<T>) {
	let actual_deposit = T::Currency::set_balance(&who, balance.saturated_into());
	assert_eq!(balance, actual_deposit);
}

fn lookup_of_account<T: Config>(
	who: T::AccountId,
) -> <<T as frame_system::Config>::Lookup as StaticLookup>::Source {
	<T as frame_system::Config>::Lookup::unlookup(who)
}

benchmarks! {
	where_clause {  where
		<T as frame_system::Config>::RuntimeOrigin: From<Origin<T>> + From<<T as frame_system::Config>::RuntimeOrigin>,
	}

	transfer {
		let schedule = Schedule::<T> {
			start: 0u32.into(),
			period: 2u32.into(),
			period_count: 3,
			per_period: T::MinReleaseTransfer::get(),
		};

		let from = T::AccountId::decode(&mut TrailingZeroInput::zeroes()).unwrap();
		whitelist_account!(from);
		let total = schedule.total_amount().unwrap();
		set_balance::<T>(&from, total * DOLLARS.into());

		let to: T::AccountId = account("to", 1, SEED);
		let to_lookup = lookup_of_account::<T>(to.clone());
	}: _(RawOrigin::Signed(from), to_lookup, schedule.clone())
	verify {
		assert_eq!(
			T::Currency::total_balance(&to),
			schedule.total_amount().unwrap()
		);
	}

	schedule_named_transfer {
		let schedule = Schedule::<T> {
			start: 0u32.into(),
			period: 2u32.into(),
			period_count: 3,
			per_period: T::MinReleaseTransfer::get(),
		};


		let from = T::AccountId::decode(&mut TrailingZeroInput::zeroes()).unwrap();
		whitelist_account!(from);
		let total = schedule.total_amount().unwrap();
		set_balance::<T>(&from, DOLLARS.into());

		let to: T::AccountId = account("to", 1, SEED);
		let to_lookup = lookup_of_account::<T>(to.clone());
		let when = 10u32.into();
	}: _(RawOrigin::Signed(from.clone()),  [1u8; 32], to_lookup, schedule.clone(), when)
	verify {
		assert_eq!(
			T::Currency::balance_on_hold(&HoldReason::TimeReleaseScheduledVesting.into(), &from),
			schedule.total_amount().unwrap());
	}

	execute_scheduled_named_transfer {
		let schedule = Schedule::<T> {
			start: 0u32.into(),
			period: 2u32.into(),
			period_count: 3,
			per_period: DOLLARS.into(),
		};

		let from = T::AccountId::decode(&mut TrailingZeroInput::zeroes()).unwrap();
		let to: T::AccountId = account("to", 1, SEED);


		whitelist_account!(from);

		// set hold balance of sender
		let total = schedule.total_amount().unwrap();
		set_balance::<T>(&from, (total * 2u32.into() * DOLLARS.into()).into());
		let _ = T::Currency::hold(&HoldReason::TimeReleaseScheduledVesting.into(), &from, total * DOLLARS.into());

		let to_lookup = lookup_of_account::<T>(to.clone());
		let origin = Origin::<T>::TimeRelease(from.clone());
	}: _(origin, [1u8; 32],to_lookup, schedule.clone())
	verify {
		assert_eq!(
			T::Currency::total_balance(&to),
			schedule.total_amount().unwrap()
		);
	}

	claim {
		let i in 1 .. T::MaxReleaseSchedules::get();

		let mut schedule = Schedule::<T> {
			start: 0u32.into(),
			period: 2u32.into(),
			period_count: 3,
			per_period: T::MinReleaseTransfer::get(),
		};

		let from = T::AccountId::decode(&mut TrailingZeroInput::zeroes()).unwrap();
		set_balance::<T>(&from, schedule.total_amount().unwrap() * BalanceOf::<T>::from(i) + DOLLARS.into());

		let to: T::AccountId = whitelisted_caller();
		let to_lookup = lookup_of_account::<T>(to.clone());

		for _ in 0..i {
			schedule.start = i.into();
			TimeReleasePallet::<T>::transfer(RawOrigin::Signed(from.clone()).into(), to_lookup.clone(), schedule.clone())?;
		}
		T::BlockNumberProvider::set_block_number(schedule.end().unwrap() + 1u32.into());
	}: _(RawOrigin::Signed(to.clone()))
	verify {
		assert_eq!(
			T::Currency::balance(&to),
			schedule.total_amount().unwrap() * BalanceOf::<T>::from(i) ,
		);
	}

	update_release_schedules {
		let i in 1 .. T::MaxReleaseSchedules::get();

		let mut schedule = Schedule::<T> {
			start: 0u32.into(),
			period: 2u32.into(),
			period_count: 3,
			per_period: T::MinReleaseTransfer::get(),
		};

		let to: T::AccountId = account("to", 0, SEED);
		set_balance::<T>(&to, schedule.total_amount().unwrap() * BalanceOf::<T>::from(i));
		let to_lookup = lookup_of_account::<T>(to.clone());

		let mut schedules = vec![];
		for _ in 0..i {
			schedule.start = i.into();
			schedules.push(schedule.clone());
		}
	}: _(RawOrigin::Root, to_lookup, schedules)
	verify {
		assert_eq!(
			T::Currency::balance(&to),
			schedule.total_amount().unwrap() * BalanceOf::<T>::from(i)
		);
	}

	cancel_scheduled_named_transfer {
		let i in 1 .. (T::MaxReleaseSchedules::get());

		let mut schedule = Schedule::<T> {
			start: 0u32.into(),
			period: 2u32.into(),
			period_count: 2,
			per_period: T::MinReleaseTransfer::get(),
		};

		let from = T::AccountId::decode(&mut TrailingZeroInput::zeroes()).unwrap();
		set_balance::<T>(&from, schedule.total_amount().unwrap() * BalanceOf::<T>::from(i) + DOLLARS.into());

		let to: T::AccountId = whitelisted_caller();
		let to_lookup = lookup_of_account::<T>(to.clone());

		for j in 0..i {
			schedule.start = i.into();

			TimeReleasePallet::<T>::schedule_named_transfer(RawOrigin::Signed(from.clone()).into(), [j as u8 ; 32], to_lookup.clone(), schedule.clone(), 4u32.into())?;
		}

		let origin = RawOrigin::Signed(from.clone());
		let schedule_name = [0u8; 32];
	}: _(origin, schedule_name)
	verify {
		ensure!(ScheduleReservedAmounts::<T>::get([0u8 ; 32]).is_none(), "Schedule not canceled");
	}


	impl_benchmark_test_suite!(
		TimeReleasePallet,
		crate::mock::ExtBuilder::build(),
		crate::mock::Test
	);
}
