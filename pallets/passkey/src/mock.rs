//! Mocks for the Passkey module.
use crate as pallet_passkey;
use crate::*;
use frame_support::{
	construct_runtime, parameter_types,
	traits::{ConstU32, ConstU64, Contains, Everything},
	weights::WeightToFee as WeightToFeeTrait,
};
use pallet_transaction_payment::FungibleAdapter;
use sp_core::{ConstU8, H256};
use sp_runtime::{
	traits::{ConvertInto, IdentityLookup},
	BuildStorage, SaturatedConversion,
};

use common_primitives::node::AccountId;

type Block = frame_system::mocking::MockBlockU32<Test>;

// Needs parameter_types! for the impls below
parameter_types! {
	pub static WeightToFee: u64 = 1;
	pub static TransactionByteFee: u64 = 1;
}

impl WeightToFeeTrait for WeightToFee {
	type Balance = u64;

	fn weight_to_fee(weight: &Weight) -> Self::Balance {
		Self::Balance::saturated_from(weight.ref_time())
			.saturating_mul(WEIGHT_TO_FEE.with(|v| *v.borrow()))
	}
}

impl WeightToFeeTrait for TransactionByteFee {
	type Balance = u64;

	fn weight_to_fee(weight: &Weight) -> Self::Balance {
		Self::Balance::saturated_from(weight.ref_time())
			.saturating_mul(TRANSACTION_BYTE_FEE.with(|v| *v.borrow()))
	}
}

construct_runtime!(
	pub enum Test
	{
		System: frame_system::{Pallet, Call, Storage, Config<T>, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Passkey: pallet_passkey::{Pallet, Storage, Call, Event<T>, ValidateUnsigned},
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage, Event<T>},
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
	type RuntimeTask = RuntimeTask;
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
	type SingleBlockMigrations = ();
	type MultiBlockMigrator = ();
	type PreInherents = ();
	type PostInherents = ();
	type PostTransactions = ();
	type ExtensionsWeightInfo = ();
}

impl pallet_transaction_payment::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = FungibleAdapter<Balances, ()>;
	type WeightToFee = WeightToFee;
	type LengthToFee = TransactionByteFee;
	type FeeMultiplierUpdate = ();
	type OperationalFeeMultiplier = ConstU8<5>;
	type WeightInfo = ();
}

impl pallet_passkey::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type RuntimeCall = RuntimeCall;
	type ConvertIntoAccountId32 = ConvertInto;
	type PasskeyCallFilter = MockPasskeyCallFilter;
	#[cfg(feature = "runtime-benchmarks")]
	type Currency = Balances;
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
	type RuntimeHoldReason = ();
	type DoneSlashHandler = ();
}

pub struct MockPasskeyCallFilter;

impl Contains<RuntimeCall> for MockPasskeyCallFilter {
	fn contains(call: &RuntimeCall) -> bool {
		match call {
			RuntimeCall::System(frame_system::Call::remark { .. }) |
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

#[cfg(feature = "runtime-benchmarks")]
pub fn new_test_ext_keystore() -> sp_io::TestExternalities {
	use sp_keystore::{testing::MemoryKeystore, KeystoreExt, KeystorePtr};
	extern crate alloc;
	use alloc::sync::Arc;

	let t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.register_extension(KeystoreExt(Arc::new(MemoryKeystore::new()) as KeystorePtr));

	ext
}
