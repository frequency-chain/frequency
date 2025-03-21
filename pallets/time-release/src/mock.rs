//! Mocks for the Time-release module.

use super::*;
use frame_support::{
	construct_runtime, parameter_types,
	traits::{
		schedule::LOWEST_PRIORITY, ConstU32, ConstU64, EitherOfDiverse, EnsureOrigin,
		EqualPrivilegeOnly, Everything,
	},
};
use frame_system::{EnsureRoot, RawOrigin};
use sp_core::H256;
use sp_runtime::{traits::IdentityLookup, BuildStorage, Perbill};

use pallet_preimage;
use pallet_scheduler;

use crate as pallet_time_release;

pub type AccountId = u128;
impl frame_system::Config for Test {
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type Block = Block;
	type BlockHashCount = ConstU32<250>;
	type BlockWeights = ();
	type BlockLength = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type RuntimeTask = RuntimeTask;
	type BaseCallFilter = Everything;
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
	type SingleBlockMigrations = ();
	type MultiBlockMigrator = ();
	type PreInherents = ();
	type PostInherents = ();
	type PostTransactions = ();
}

type Balance = u64;

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = frame_system::Pallet<Test>;
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
	type FreezeIdentifier = RuntimeFreezeReason;
	type MaxFreezes = ConstU32<1>;
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
}

impl pallet_preimage::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type Currency = ();
	type ManagerOrigin = EnsureRoot<AccountId>;
	type Consideration = ();
}

parameter_types! {
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(
			Weight::from_parts(2_000_000_000_000, u64::MAX),
		);
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
		BlockWeights::get().max_block;
}

pub struct EnsureTimeReleaseOrigin;

impl EnsureOrigin<RuntimeOrigin> for EnsureTimeReleaseOrigin {
	type Success = AccountId;

	fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
		match o.clone().into() {
			Ok(pallet_time_release::Origin::<Test>::TimeRelease(who)) => Ok(who),
			_ => Err(o),
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<RuntimeOrigin, ()> {
		Ok(RuntimeOrigin::root())
	}
}

impl pallet_scheduler::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type MaximumWeight = MaximumSchedulerWeight;
	/// Origin to schedule or cancel calls
	/// Set to Root or a simple majority of the Frequency Council
	type ScheduleOrigin = EitherOfDiverse<EnsureTimeReleaseOrigin, EnsureRoot<AccountId>>;
	type MaxScheduledPerBlock = ConstU32<50>;
	type WeightInfo = common_runtime::weights::pallet_scheduler::SubstrateWeight<Test>;
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type Preimages = Preimage;
}

pub struct EnsureAliceOrBob;
impl EnsureOrigin<RuntimeOrigin> for EnsureAliceOrBob {
	type Success = AccountId;

	fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
		Into::<Result<RawOrigin<AccountId>, RuntimeOrigin>>::into(o).and_then(|o| match o {
			RawOrigin::Signed(ALICE) => Ok(ALICE),
			RawOrigin::Signed(BOB) => Ok(BOB),
			r => Err(RuntimeOrigin::from(r)),
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<RuntimeOrigin, ()> {
		let zero_account_id =
			AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
				.map_err(|_| ())?;

		Ok(RuntimeOrigin::from(RawOrigin::Signed(zero_account_id)))
	}
}

// Needs parameter_types! for the impls below
parameter_types! {
	pub static MockBlockNumberProvider: u32 = 0;
}

impl BlockNumberProvider for MockBlockNumberProvider {
	type BlockNumber = u32;

	fn current_block_number() -> Self::BlockNumber {
		Self::get()
	}
}

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeHoldReason = RuntimeHoldReason;
	type Currency = PalletBalances;
	type MinReleaseTransfer = ConstU64<1u64>;
	type TransferOrigin = EnsureAliceOrBob;
	type WeightInfo = ();
	type MaxReleaseSchedules = ConstU32<50>;
	type BlockNumberProvider = MockBlockNumberProvider;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type Balance = Balance;
	type RuntimeCall = RuntimeCall;
	type SchedulerProvider = SchedulerProvider;
	type TimeReleaseOrigin = EnsureTimeReleaseOrigin;
}

type Block = frame_system::mocking::MockBlockU32<Test>;

construct_runtime!(
	pub enum Test
	{
		System: frame_system::{Pallet, Call, Storage, Config<T>, Event<T>},
		TimeRelease: pallet_time_release::{Pallet, Storage, Call, Event<T>, Config<T>, FreezeReason, HoldReason, Origin<T>},
		PalletBalances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>, HoldReason},
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>}
	}
);

pub const ALICE: AccountId = 0;
pub const BOB: AccountId = 2;
pub const CHARLIE: AccountId = 3;
pub const DAVE: AccountId = 4;

pub const ALICE_BALANCE: u64 = 100;
pub const CHARLIE_BALANCE: u64 = 30;
pub const DAVE_BALANCE: u64 = 200;

pub struct SchedulerProvider;

impl SchedulerProviderTrait<RuntimeOrigin, u32, RuntimeCall> for SchedulerProvider {
	fn schedule(
		origin: RuntimeOrigin,
		id: ScheduleName,
		when: u32,
		call: Box<RuntimeCall>,
	) -> Result<(), DispatchError> {
		Scheduler::schedule_named(origin, id, when, None, LOWEST_PRIORITY, call)?;

		Ok(())
	}

	fn cancel(origin: RuntimeOrigin, id: ScheduleName) -> Result<(), DispatchError> {
		Scheduler::cancel_named(origin, id)?;

		Ok(())
	}
}

// Remove capacity on_initialize, needed to emulate pre-existing block height
pub fn run_to_block(n: u32) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			System::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		Scheduler::on_initialize(System::block_number());
		System::on_initialize(System::block_number());
	}
}

#[derive(Default)]
pub struct ExtBuilder;

impl ExtBuilder {
	pub fn build() -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

		MockBlockNumberProvider::set(0);

		pallet_balances::GenesisConfig::<Test> {
			balances: vec![
				(ALICE, ALICE_BALANCE),
				(CHARLIE, CHARLIE_BALANCE),
				(DAVE, DAVE_BALANCE),
			],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		pallet_time_release::GenesisConfig::<Test> {
			_config: Default::default(),
			schedules: vec![
				// who, start, period, period_count, per_period
				(CHARLIE, 2, 3, 1, 5),
				(CHARLIE, 2 + 3, 3, 3, 5),
			],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		t.into()
	}
}
