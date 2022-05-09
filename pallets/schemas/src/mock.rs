use frame_support::{
	parameter_types,
	traits::{ConstU16, ConstU32, ConstU64},
};
use frame_system;
use sp_core::{ed25519, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup, Verify},
};

use common_primitives::schema::SchemaId;

use crate as pallet_schemas;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
pub type Signature = ed25519::Signature;
pub type AccountId = <Signature as Verify>::Signer;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		SchemasPallet: pallet_schemas::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const MaxSchemaRegistrations: SchemaId = 64_000;
}

impl pallet_schemas::Config for Test {
	type Event = Event;
	type Public = AccountId;
	type Signature = Signature;
	// type CallHasher = Keccak256;
	type WeightInfo = ();
	type MinSchemaSize = ConstU32<5>;
	type MaxSchemaSize = ConstU32<100>;
	type MaxSchemaRegistrations = MaxSchemaRegistrations;
}

impl frame_system::Config for Test {
	type AccountData = ();
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type Call = Call;
	type DbWeight = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext: sp_io::TestExternalities =
		frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into();
	ext.execute_with(|| System::set_block_number(1));
	ext
}
