use crate as pallet_stateful;
use common_primitives::{
	msa::{
		Delegation, DelegationValidator, DelegatorId, MessageSourceId, MsaLookup, MsaValidator,
		ProviderId, ProviderLookup, SchemaGrantValidator,
	},
	schema::*,
};

use frame_support::{
	dispatch::DispatchResult,
	parameter_types,
	traits::{ConstU16, ConstU64, OnFinalize, OnInitialize},
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	DispatchError,
};
use std::fmt::Formatter;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		StatefulMessageStoragePallet: pallet_stateful::{Pallet, Call, Storage, Event<T>},
	}
);

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
	pub const MaxItemizedPageSizeBytes: u32 = 1024;
	pub static MaxPaginatedPageSizeBytes: u32 = 1024;
	pub const MaxItemizedBlobSizeBytes: u32 = 64;
	pub const MaxPaginatedPageCount: u8 = 32;
}

impl Clone for MaxPaginatedPageCount {
	fn clone(&self) -> Self {
		MaxPaginatedPageCount {}
	}
}

impl Eq for MaxPaginatedPageCount {
	fn assert_receiver_is_total_eq(&self) -> () {}
}

impl PartialEq for MaxPaginatedPageCount {
	fn eq(&self, _other: &Self) -> bool {
		true
	}
}

impl sp_std::fmt::Debug for MaxPaginatedPageCount {
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

impl Clone for MaxItemizedPageSizeBytes {
	fn clone(&self) -> Self {
		MaxItemizedPageSizeBytes {}
	}
}

impl Eq for MaxItemizedPageSizeBytes {
	fn assert_receiver_is_total_eq(&self) -> () {}
}

impl PartialEq for MaxItemizedPageSizeBytes {
	fn eq(&self, _other: &Self) -> bool {
		true
	}
}

impl sp_std::fmt::Debug for MaxItemizedPageSizeBytes {
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

impl Clone for MaxPaginatedPageSizeBytes {
	fn clone(&self) -> Self {
		MaxPaginatedPageSizeBytes {}
	}
}

impl Eq for MaxPaginatedPageSizeBytes {
	fn assert_receiver_is_total_eq(&self) -> () {}
}

impl PartialEq for MaxPaginatedPageSizeBytes {
	fn eq(&self, _other: &Self) -> bool {
		true
	}
}

impl sp_std::fmt::Debug for MaxPaginatedPageSizeBytes {
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

impl Clone for MaxItemizedBlobSizeBytes {
	fn clone(&self) -> Self {
		MaxItemizedBlobSizeBytes {}
	}
}

impl Eq for MaxItemizedBlobSizeBytes {
	fn assert_receiver_is_total_eq(&self) -> () {}
}

impl PartialEq for MaxItemizedBlobSizeBytes {
	fn eq(&self, _other: &Self) -> bool {
		true
	}
}

impl sp_std::fmt::Debug for MaxItemizedBlobSizeBytes {
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

impl pallet_stateful::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type MaxItemizedBlobSizeBytes = MaxItemizedBlobSizeBytes;
	type MaxPaginatedPageCount = MaxPaginatedPageCount;
	type MaxItemizedPageSizeBytes = MaxItemizedPageSizeBytes;
	type MaxPaginatedPageSizeBytes = MaxPaginatedPageSizeBytes;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
