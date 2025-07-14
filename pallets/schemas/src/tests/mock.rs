use frame_support::{
	assert_ok,
	dispatch::RawOrigin,
	traits::{ConstU16, ConstU32, EitherOfDiverse},
	weights::{Weight, WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial},
	BoundedVec,
};
use frame_system::EnsureRoot;
use parity_scale_codec::MaxEncodedLen;

use common_primitives::node::AccountId;
use common_runtime::constants::DAYS;
use pallet_collective::{self, Members};
use smallvec::smallvec;
use sp_core::{parameter_types, Encode, H256};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32, BuildStorage, DispatchError, Perbill,
};

use crate as pallet_schemas;

type Block = frame_system::mocking::MockBlockU32<Test>;

frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		SchemasPallet: pallet_schemas::{Pallet, Call, Storage, Event<T>},
		Council: pallet_collective::<Instance1>::{Pallet, Call, Config<T,I>, Storage, Event<T>, Origin<T>},
	}
);

// See https://paritytech.github.io/substrate/master/pallet_collective/index.html for
// the descriptions of these configs.

parameter_types! {
	pub BlockWeights: frame_system::limits::BlockWeights =
	frame_system::limits::BlockWeights::simple_max(Weight::MAX);
	pub MaxProposalWeight: frame_support::weights::Weight  = sp_runtime::Perbill::from_percent(50) * BlockWeights::get().max_block;
	pub const SchemaModelMaxBytesBoundedVecLimit :u32 = 65_500;
}

impl Encode for SchemaModelMaxBytesBoundedVecLimit {}

impl MaxEncodedLen for SchemaModelMaxBytesBoundedVecLimit {
	fn max_encoded_len() -> usize {
		u32::max_encoded_len()
	}
}

pub type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Test {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = ConstU32<{ 5 * DAYS }>;
	type MaxProposals = ConstU32<25>;
	type MaxMembers = ConstU32<10>;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = ();

	type SetMembersOrigin = frame_system::EnsureRoot<Self::AccountId>;
	type MaxProposalWeight = MaxProposalWeight;
	type DisapproveOrigin = EnsureRoot<AccountId>;
	type KillOrigin = EnsureRoot<AccountId>;
	type Consideration = ();
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
		let members = Members::<Test, CouncilCollective>::get();
		let threshold: u32 = ((members.len() / 2) + 1) as u32;
		let length_bound: u32 = proposal.using_encoded(|p| p.len() as u32);
		Council::do_propose_proposed(who, threshold, proposal, length_bound)
	}

	#[cfg(any(feature = "runtime-benchmarks", feature = "test"))]
	fn proposal_count() -> u32 {
		use pallet_collective::ProposalCount;
		ProposalCount::<Test, CouncilCollective>::get()
	}
}

impl pallet_schemas::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type MinSchemaModelSizeBytes = ConstU32<8>;
	// a very high limit on incoming schema size, expected to be much higher than what
	// is actually allowed.
	type SchemaModelMaxBytesBoundedVecLimit = SchemaModelMaxBytesBoundedVecLimit;
	type MaxSchemaRegistrations = MaxSchemaRegistrations;
	type MaxSchemaSettingsPerSchema = ConstU32<2>;
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
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeTask = RuntimeTask;
	type Block = Block;
	type BlockHashCount = ConstU32<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type SingleBlockMigrations = ();
	type MultiBlockMigrator = ();
	type PreInherents = ();
	type PostInherents = ();
	type PostTransactions = ();
	type ExtensionsWeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext: sp_io::TestExternalities =
		frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into();
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

/// creates a bounded vec schema
pub fn create_bounded_schema_vec(
	from_string: &str,
) -> BoundedVec<u8, <Test as crate::Config>::SchemaModelMaxBytesBoundedVecLimit> {
	let fields_vec = Vec::from(from_string.as_bytes());
	BoundedVec::try_from(fields_vec).unwrap()
}

/// sets max schema size
pub fn sudo_set_max_schema_size() {
	assert_ok!(SchemasPallet::set_max_schema_model_bytes(RawOrigin::Root.into(), 70));
}

/// struct which encapsulates a test case and expected result
pub struct TestCase<T> {
	pub input: &'static str,
	pub expected: T,
}
