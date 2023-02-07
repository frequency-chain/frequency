use crate::{
	self as pallet_msa,
	mock::{generate_test_signature, new_test_ext, run_to_block},
	Error,
};
use common_primitives::node::{AccountId, Hash};
use frame_support::{
	assert_noop, assert_ok,
	dispatch::{DispatchErrorWithPostInfo, DispatchResultWithPostInfo, PostDispatchInfo},
	parameter_types,
	traits::{ConstU16, ConstU32, ConstU64, EitherOfDiverse, Everything},
	weights::Weight,
};
use frame_system::{pallet_prelude::OriginFor, EnsureRoot, EnsureSigned};
use pallet_collective;

use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, ConvertInto, IdentityLookup},
};

pub use common_runtime::constants::*;
pub use pallet_msa::Call as MsaCall;

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
		Msa: pallet_msa::{Pallet, Call, Storage, Event<T>},
		Schemas: pallet_schemas::{Pallet, Call, Storage, Event<T>},
		Council: pallet_collective::<Instance1>::{Pallet, Call, Config<T,I>, Storage, Event<T>, Origin<T>},
	}
);

impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
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
	type MaxConsumers = ConstU32<16>;
}

impl pallet_schemas::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type MinSchemaModelSizeBytes = ConstU32<10>;
	type SchemaModelMaxBytesBoundedVecLimit = ConstU32<10>;
	type MaxSchemaRegistrations = ConstU16<10>;
}

parameter_types! {
	pub const MaxPublicKeysPerMsa: u8 = 255;
	pub const MaxProviderNameSize: u32 = 16;
	pub const MaxSchemas: u32 = 5;
}

parameter_types! {
	pub const MaxSchemaGrantsPerDelegation: u32 = 30;
}

impl Clone for MaxSchemaGrantsPerDelegation {
	fn clone(&self) -> Self {
		MaxSchemaGrantsPerDelegation {}
	}
}

impl Eq for MaxSchemaGrantsPerDelegation {
	fn assert_receiver_is_total_eq(&self) -> () {}
}

impl PartialEq for MaxSchemaGrantsPerDelegation {
	fn eq(&self, _other: &Self) -> bool {
		true
	}
}

impl sp_std::fmt::Debug for MaxSchemaGrantsPerDelegation {
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

// See https://paritytech.github.io/substrate/master/pallet_collective/index.html for
// the descriptions of these configs.
type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Test {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = ();
}
/// Interface to collective pallet to propose a proposal.
pub struct CouncilProposalProvider;
impl pallet_msa::ProposalProvider<AccountId, RuntimeCall> for CouncilProposalProvider {
	fn propose(
		origin: OriginFor<T>,
		threshold: u32,
		proposal: Box<RuntimeCall>,
		length_bound: u32,
	) -> DispatchResultWithPostInfo {
		Council::propose(origin, threshold, proposal, length_bound)
	}

	fn vote(
		origin: OriginFor<T>,
		proposal: Hash,
		index: u32,
		approve: bool,
	) -> DispatchResultWithPostInfo {
		Council::vote(origin, proposal, index, approve)
	}

	fn close(
		proposal_hash: Hash,
		index: u32,
		length_bound: u32,
	) -> Result<PostDispatchInfo, DispatchErrorWithPostInfo> {
		Council::do_close(proposal_hash, index, Weight::zero(), length_bound)
	}

	fn proposal_of(hash: Hash) -> Option<RuntimeCall> {
		Council::proposal_of(hash)
	}
}

#[cfg(not(feature = "frequency"))]
type MsaCreateProviderOrigin = EnsureSigned<AccountId>;
#[cfg(feature = "frequency")]
type MsaCreateProviderOrigin = EnsureNever<AccountId>;

type MsaCreateProviderViaGovernanceOrigin = EitherOfDiverse<
	EnsureRoot<AccountId>,
	pallet_collective::EnsureMembers<AccountId, CouncilCollective, 1>,
>;

impl pallet_msa::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type ConvertIntoAccountId32 = ConvertInto;
	type MaxPublicKeysPerMsa = MaxPublicKeysPerMsa;
	type MaxSchemaGrantsPerDelegation = MaxSchemaGrantsPerDelegation;
	type MaxProviderNameSize = MaxProviderNameSize;
	type SchemaValidator = Schemas;
	type MortalityWindowSize = ConstU32<10>;
	type MaxSignaturesPerBucket = ConstU32<20>;
	type NumberOfBuckets = ConstU32<10>;
	/// This MUST ALWAYS be MaxSignaturesPerBucket * NumberOfBuckets.
	type MaxSignaturesStored = ConstU32<200>;
	// The proposal type
	type Proposal = RuntimeCall;
	// The Council proposal provider interface
	type ProposalProvider = CouncilProposalProvider;
	type CreateProviderOrigin = MsaCreateProviderOrigin;
	// The origin that is allowed to create providers via governance
	type CreateProviderViaGovernanceOrigin = MsaCreateProviderViaGovernanceOrigin;
}

#[test]
fn audit_replay_scenario_fails() {
	new_test_ext().execute_with(|| {
		let current_block = 9;
		System::set_block_number(current_block);

		let sig1 = &generate_test_signature();
		let mortality: u64 = 13;
		assert_ok!(Msa::register_signature(sig1, mortality));

		assert_noop!(
			Msa::register_signature(sig1, mortality),
			Error::<Test>::SignatureAlreadySubmitted,
		);

		run_to_block(10);
		assert_noop!(
			Msa::register_signature(sig1, mortality),
			Error::<Test>::SignatureAlreadySubmitted,
		);

		run_to_block(mortality - 1);
		assert_noop!(
			Msa::register_signature(sig1, mortality),
			Error::<Test>::SignatureAlreadySubmitted,
		);

		run_to_block(mortality);
		assert_noop!(Msa::register_signature(sig1, mortality), Error::<Test>::ProofHasExpired,);
	})
}
