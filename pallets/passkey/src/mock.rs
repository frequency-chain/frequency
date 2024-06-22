//! Mocks for the Passkey module.

use super::*;
use frame_support::{
	construct_runtime,
	traits::{ConstU32, ConstU64, Everything},
};
use sp_core::H256;
use sp_runtime::{traits::IdentityLookup, BuildStorage};

use crate as pallet_passkey;

use common_primitives::node::AccountId;

type Block = frame_system::mocking::MockBlockU32<Test>;

construct_runtime!(
	pub enum Test
	{
		System: frame_system::{Pallet, Call, Storage, Config<T>, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Passkey: pallet_passkey::{Pallet, Storage, Call, Event<T>},
	}
);

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
	type BaseCallFilter = Everything;
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type RuntimeCall = RuntimeCall;
	type PasskeyCallFilter = MockPasskeyCallFilter;
}

impl pallet_balances::Config for Test {
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type Balance = u64;
	type MaxLocks = ();
	type WeightInfo = ();
	type ReserveIdentifier = [u8; 8];
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type MaxReserves = ();
	type FreezeIdentifier = RuntimeFreezeReason;
	type RuntimeFreezeReason = ();
	type MaxFreezes = ConstU32<1>;
	type MaxHolds = ConstU32<0>;
	type RuntimeHoldReason = ();
}

pub struct MockPasskeyCallFilter;

impl Contains<RuntimeCall> for MockPasskeyCallFilter {
	fn contains(call: &RuntimeCall) -> bool {
		match call {
			RuntimeCall::Balances(pallet_balances::Call::transfer_keep_alive { .. }) |
			RuntimeCall::Balances(pallet_balances::Call::transfer_allow_death { .. }) |
			RuntimeCall::Balances(pallet_balances::Call::transfer_all { .. }) => true,
			_ => false,
		}
	}
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext: sp_io::TestExternalities =
		frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into();
	ext.execute_with(|| System::set_block_number(1));
	ext
}
