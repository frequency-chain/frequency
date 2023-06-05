use frame_support::{
	dispatch::DispatchError,
	traits::{ConstU16, ConstU32, EitherOfDiverse},
	weights::{WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial},
};
use frame_system::EnsureRoot;

use common_primitives::node::{AccountId, Header};
pub use common_runtime::constants::*;
use pallet_collective;
use smallvec::smallvec;
use sp_core::{Encode, H256};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32, Perbill,
};

use crate as pallet_schemas;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		SchemasPallet: pallet_schemas::{Pallet, Call, Storage, Event<T>},
		Council: pallet_collective::<Instance1>::{Pallet, Call, Config<T,I>, Storage, Event<T>, Origin<T>},
	}
);

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
	type SetMembersOrigin = frame_system::EnsureRoot<AccountId32>;
}

pub type MaxSchemaRegistrations = ConstU16<64_000>;

pub struct WeightToFee;

impl WeightToFeePolynomial for WeightToFee {
	type Balance = u64;

	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		smallvec![WeightToFeeCoefficient {
			degree: 1,
			coeff_frac: Perbill::zero(),
			coeff_integer: 1,
			negative: false,
		}]
	}
}

/// Interface to collective pallet to propose a proposal.
pub struct CouncilProposalProvider;

impl pallet_schemas::ProposalProvider<AccountId, RuntimeCall> for CouncilProposalProvider {
	fn propose(
		who: AccountId,
		threshold: u32,
		proposal: Box<RuntimeCall>,
	) -> Result<(u32, u32), DispatchError> {
		let length_bound: u32 = proposal.using_encoded(|p| p.len() as u32);
		Council::do_propose_proposed(who, threshold, proposal, length_bound)
	}

	fn propose_with_simple_majority(
		who: AccountId,
		proposal: Box<RuntimeCall>,
	) -> Result<(u32, u32), DispatchError> {
		let threshold: u32 = ((Council::members().len() / 2) + 1) as u32;
		let length_bound: u32 = proposal.using_encoded(|p| p.len() as u32);
		Council::do_propose_proposed(who, threshold, proposal, length_bound)
	}

	#[cfg(any(feature = "runtime-benchmarks", feature = "test"))]
	fn proposal_count() -> u32 {
		Council::proposal_count()
	}
}

impl pallet_schemas::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type MinSchemaModelSizeBytes = ConstU32<8>;
	// a very high limit on incoming schema size, expected to be much higher than what
	// is actually allowed.
	type SchemaModelMaxBytesBoundedVecLimit = ConstU32<65_500>;
	type MaxSchemaRegistrations = MaxSchemaRegistrations;
	type MaxSchemaSettingsPerSchema = ConstU32<1>;
	// The proposal type
	type Proposal = RuntimeCall;
	// The Council proposal provider interface
	type ProposalProvider = CouncilProposalProvider;
	// The origin that is allowed to create schemas via governance
	// It has to be this way so benchmarks will pass in CI.
	type CreateSchemaViaGovernanceOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>,
	>;
}

impl frame_system::Config for Test {
	type AccountData = ();
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type RuntimeCall = RuntimeCall;
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type Index = u64;
	type BlockNumber = u32;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU32<250>;
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

/// Create and return a simple test AccountId32 constructed with the desired integer.
pub fn test_public(n: u8) -> AccountId32 {
	AccountId32::new([n; 32])
}

/// Create and return a simple signed origin from a test_public constructed with the desired integer,
/// for passing to an extrinsic call
pub fn test_origin_signed(n: u8) -> RuntimeOrigin {
	RuntimeOrigin::signed(test_public(n))
}
