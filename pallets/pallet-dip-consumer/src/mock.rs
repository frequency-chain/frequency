use crate::{self as pallet_dip_consumer, };

use frame_support::{parameter_types, traits::Everything};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    generic,
	traits::{BlakeTwo256, IdentityLookup, ConstU32, ConstU16}, AccountId32,
};

use cumulus_pallet_xcm::Origin;

use std::fmt::Formatter;

pub type Header = generic::Header<u32, BlakeTwo256>;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Event<T>},
        DipConsumer: pallet_dip_consumer::{Pallet, Call, Storage, Event<T>}
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
	type BlockNumber = u32;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU32<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}



pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

impl pallet_dip_consumer::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    // The identifier of a subject, e.g., a DID.
    type Identifier = AccountId32;
    // The details stored in this pallet associated with any given subject.
    type IdentityDetails = u128;
    /// The proof users must provide to operate with their higher-level
    /// identity. Depending on the use cases, this proof can contain
    /// heterogeneous bits of information that the proof verifier will
    /// utilize. For instance, a proof could contain both a Merkle proof and
    /// a DID signature.
    type Proof = ();
    /// The type of the committed proof digest used as the basis for
    /// verifying identity proofs.
    type ProofDigest = sp_core::H256;

	// The overarching runtime call type.
    // type RuntimeCall = RuntimeCall;

	// The overarching runtime origin type.
    // type RuntimeOrigin = RuntimeOrigin;
}