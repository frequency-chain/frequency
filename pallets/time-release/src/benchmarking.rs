#![allow(clippy::unwrap_used)]
use super::*;

use crate::Pallet as TimeReleasePallet;

use frame_benchmarking::{account, benchmarks, whitelist_account, whitelisted_caller};
use frame_system::{pallet_prelude::BlockNumberFor, RawOrigin};
use sp_runtime::{traits::TrailingZeroInput, SaturatedConversion};
use sp_std::prelude::*;

pub const DOLLARS: u32 = 10u32.saturating_pow(8u32);

pub use crate::types::ReleaseSchedule;
pub type Schedule<T> = ReleaseSchedule<BlockNumberFor<T>, BalanceOf<T>>;

const SEED: u32 = 0;

// TODO: this function is duplicated in pallets/time-release/src/mock.rs
fn set_balance<T: Config>(who: &T::AccountId, balance: BalanceOf<T>) {
	let _ = T::Currency::mint_into(&who, balance.saturated_into());
	assert_eq!(
		T::Currency::balance(who).saturated_into::<u64>(),
		balance.saturated_into::<u64>() + 100u64
	);
}

fn lookup_of_account<T: Config>(
	who: T::AccountId,
) -> <<T as frame_system::Config>::Lookup as StaticLookup>::Source {
	<T as frame_system::Config>::Lookup::unlookup(who)
}

benchmarks! {
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

	impl_benchmark_test_suite!(
		TimeReleasePallet,
		crate::mock::ExtBuilder::build(),
		crate::mock::Test
	);
}
