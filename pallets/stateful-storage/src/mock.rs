use crate as pallet_stateful_storage;
use common_primitives::{
	msa::{MessageSourceId, MsaLookup, MsaValidator},
	schema::*,
};

use frame_support::{
	parameter_types,
	traits::{ConstU16, ConstU64},
};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	DispatchError,
};

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
		StatefulStoragePallet: pallet_stateful_storage::{Pallet, Call, Storage, Event<T>},
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
	pub const MaxPaginatedPageSizeBytes: u32 = 1024;
	pub const MaxItemizedBlobSizeBytes: u32 = 64;
	pub const MaxPaginatedPageCount: u16 = 32;
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

pub struct MsaInfoHandler;
impl MsaLookup for MsaInfoHandler {
	type AccountId = u64;

	fn get_msa_id(key: &Self::AccountId) -> Option<MessageSourceId> {
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

pub struct SchemaHandler;
impl SchemaProvider<u16> for SchemaHandler {
	fn get_schema_by_id(schema_id: SchemaId) -> Option<SchemaResponse> {
		Some(SchemaResponse {
			schema_id,
			model: r#"schema"#.to_string().as_bytes().to_vec(),
			model_type: ModelType::AvroBinary,
			payload_location: PayloadLocation::OnChain,
		})
	}
}

impl pallet_stateful_storage::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type MsaInfoProvider = MsaInfoHandler;
	type SchemaProvider = SchemaHandler;
	type WeightInfo = ();
	type MaxItemizedBlobSizeBytes = MaxItemizedBlobSizeBytes;
	type MaxPaginatedPageCount = MaxPaginatedPageCount;
	type MaxItemizedPageSizeBytes = MaxItemizedPageSizeBytes;
	type MaxPaginatedPageSizeBytes = MaxPaginatedPageSizeBytes;

	/// A set of helper functions for benchmarking.
	#[cfg(feature = "runtime-benchmarks")]
	type MsaBenchmarkHelper = ();
	#[cfg(feature = "runtime-benchmarks")]
	type SchemaBenchmarkHelper = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

pub fn get_msa_from_account(account_id: u64) -> u64 {
	account_id + 100
}
