//! Mocks for the Time-release module.

use super::*;
use frame_support::{
	construct_runtime,
	traits::{ConstU32, Everything},
};
use sp_core::H256;
use sp_runtime::{
	traits::{ConvertInto, IdentityLookup},
	BuildStorage,
};

use crate as pallet_passkey;

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
	type AccountData = ();
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
	type ConvertIntoAccountId32 = ConvertInto;
}

type Block = frame_system::mocking::MockBlockU32<Test>;

construct_runtime!(
	pub enum Test
	{
		System: frame_system::{Pallet, Call, Storage, Config<T>, Event<T>},
		Passkey: pallet_passkey::{Pallet, Storage, Call, Event<T>},
	}
);

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext: sp_io::TestExternalities =
		frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into();
	ext.execute_with(|| System::set_block_number(1));
	ext
}
