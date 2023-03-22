use crate as pallet_handles;
use common_primitives::msa::{MessageSourceId, MsaLookup, MsaValidator};
use frame_support::{
	traits::{ConstU16, ConstU32, ConstU64},
};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	DispatchError,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub struct MsaInfoHandler;

impl MsaLookup for MsaInfoHandler {
	type AccountId = u64;

	fn get_msa_id(key: &Self::AccountId) -> Option<MessageSourceId> {
		if *key == 1000 {
			return None
		}
		if *key == 2000 {
			return Some(2000 as MessageSourceId)
		}
		Some(get_msa_from_account(*key) as MessageSourceId)
	}
}

impl MsaValidator for MsaInfoHandler {
	type AccountId = u64;

	fn ensure_valid_msa_key(key: &Self::AccountId) -> Result<MessageSourceId, DispatchError> {
		if *key == 1000 {
			return Err(DispatchError::Other("some error"))
		}
		if *key == 2000 {
			return Ok(2000)
		}

		Ok(get_msa_from_account(*key))
	}
}

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		Handles: pallet_handles::{Pallet, Call, Storage, Event<T>},
	}
);

impl frame_system::Config for Test {
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

impl pallet_handles::Config for Test {
	type RuntimeEvent = RuntimeEvent;

	/// Weight information for extrinsics in this pallet.
	// type WeightInfo: WeightInfo;

	/// A type that will supply MSA related information
	type MsaInfoProvider = MsaInfoHandler;

	/// The minimum suffix value
	type HandleSuffixMin = ConstU32<2>;

	/// The maximum suffix value
	type HandleSuffixMax = ConstU32<30>;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}

pub fn get_msa_from_account(account_id: u64) -> u64 {
	account_id + 100
}