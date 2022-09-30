use crate::{
	AccountId, Balance, Balances, BlockNumber, ORMLMaxVestingSchedules, Runtime, System, Vesting,
};

pub const UNITS: Balance = 1_000_000_000_000;
pub const DOLLARS: Balance = UNITS * 100;
pub const CENTS: Balance = DOLLARS / 100;
pub const NATIVE_EXISTENTIAL_DEPOSIT: Balance = CENTS;

use frame_benchmarking::{account, whitelisted_caller};
use frame_support::traits::{Currency, Imbalance};
use frame_system::RawOrigin;
use sp_std::prelude::*;

use sp_runtime::traits::{AccountIdConversion, SaturatedConversion, StaticLookup};

use orml_benchmarking::runtime_benchmarks;
use orml_vesting::VestingSchedule;

pub type Schedule = VestingSchedule<BlockNumber, Balance>;
const SEED: u32 = 0;

fn get_vesting_account() -> AccountId {
	crate::VestingPalletId::get().into_account_truncating()
}

fn lookup_of_account(
	who: AccountId,
) -> <<Runtime as frame_system::Config>::Lookup as StaticLookup>::Source {
	<Runtime as frame_system::Config>::Lookup::unlookup(who)
}

fn set_balance(who: &AccountId, balance: Balance) {
	let deposit_result = <Balances as Currency<_>>::deposit_creating(who, balance.saturated_into());

	let actual_deposit = deposit_result.peek();
	assert_eq!(balance, actual_deposit);
}

runtime_benchmarks! {
	{ Runtime, orml_vesting }

	vested_transfer {
		let schedule = Schedule {
			start: 0,
			period: 2,
			period_count: 3,
			per_period: NATIVE_EXISTENTIAL_DEPOSIT,
		};

		let from: AccountId = get_vesting_account();

		let total = schedule.total_amount().unwrap();
		set_balance(&from, total + UNITS);

		let to: AccountId = account("to", 0, SEED);
		let to_lookup = lookup_of_account(to.clone());
	}: _(RawOrigin::Root, to_lookup, schedule.clone())
	verify {
		assert_eq!(
			<Balances as Currency<_>>::total_balance(&to),
			schedule.total_amount().unwrap()
		);
	}

	claim {
		let i in 1 .. ORMLMaxVestingSchedules::get();

		let mut schedule = Schedule {
			start: 0,
			period: 2,
			period_count: 3,
			per_period: NATIVE_EXISTENTIAL_DEPOSIT,
		};

		let from: AccountId = get_vesting_account();
		set_balance(&from, schedule.total_amount().unwrap() * i as u128 + UNITS);

		let to: AccountId = whitelisted_caller();
		let to_lookup = lookup_of_account(to.clone());

		for _ in 0..i {
			schedule.start = i;
			Vesting::vested_transfer(RawOrigin::Root.into(), to_lookup.clone(), schedule.clone())?;
		}
		System::set_block_number(schedule.end().unwrap() + 1u32);
	}: _(RawOrigin::Signed(to.clone()))
	verify {
		assert_eq!(
			<Balances as Currency<_>>::free_balance(&to),
			schedule.total_amount().unwrap() * i as u128,
		);
	}

	update_vesting_schedules {
		let i in 1 .. ORMLMaxVestingSchedules::get();

		let mut schedule = Schedule {
			start: 0,
			period: 2,
			period_count: 3,
			per_period: NATIVE_EXISTENTIAL_DEPOSIT,
		};

		let to: AccountId = account("to", 0, SEED);
		set_balance(&to, schedule.total_amount().unwrap() * i as u128);
		let to_lookup = lookup_of_account(to.clone());

		let mut schedules = vec![];
		for _ in 0..i {
			schedule.start = i;
			schedules.push(schedule.clone());
		}
	}: _(RawOrigin::Root, to_lookup, schedules)
	verify {
		assert_eq!(
			<Balances as Currency<_>>::free_balance(&to),
			schedule.total_amount().unwrap() * i as u128
		);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use orml_benchmarking::impl_benchmark_test_suite;

	pub fn new_test_ext() -> sp_io::TestExternalities {
		frame_system::GenesisConfig::default()
			.build_storage::<crate::Runtime>()
			.unwrap()
			.into()
	}

	impl_benchmark_test_suite!(new_test_ext(),);
}
