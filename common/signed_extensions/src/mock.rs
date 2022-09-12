use frame_support::{
	ord_parameter_types, parameter_types,
	traits::{ConstU32, ConstU64, EqualPrivilegeOnly},
};
use frame_support::dispatch::RawOrigin;
use frame_support::traits::EnsureOrigin;
use frame_system as system;
use frame_system::{Account, EnsureRoot};
use pallet_balances;
use pallet_democracy;
use pallet_collective;
use pallet_collective::EnsureMember;
use sp_core::crypto::AccountId32;
use sp_std::convert::From;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup, SignedExtension },
	transaction_validity::TransactionValidity
};
use crate::democracy::VerifyVoter;

pub type BlockNumber = u64;
pub type AccountId = AccountId32;

pub type Block = frame_system::mocking::MockBlock<Test>;
pub type MemberEnsurer = EnsureMember<AccountId, MTHolders>;

pub type SignedExtra = (VerifyVoter.new(&MemberEnsurer));

pub type UncheckedExtrinsic = sp_runtime::generic::UncheckedExtrinsic<AccountId, Call, Signature, SignedExtra>;
// //pub type UncheckedExtrinsic = sp_runtime::generic::UncheckedExtrinsic<u32, u64, Call, ()>;
// type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		Preimage: pallet_preimage,
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>},
		DefaultCollective: pallet_collective::{Pallet, Call, Event<T>, Origin<T>, Config<T>},
		MTHolders: pallet_collective::<Instance1>::{Pallet, Call, Event<T>, Origin<T>, Config<T>},
		Democracy: pallet_democracy::{Pallet, Call, Storage, Config<T>, Event<T>},
	}
);

impl system::Config for Test {
	type AccountId = AccountId;
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const MILLISECS_PER_BLOCK: u64 = 12000;

ord_parameter_types! {
	pub const One: u64 = 1;
	pub const Two: u64 = 2;
}

pub type Balance = u64;
pub const UNIT: Balance = 1_000_000_000_000;
pub const MICROUNIT: Balance = 1_000_000;

parameter_types! {
	pub const EnactmentPeriod: BlockNumber = 30 * 24 * 60 * MINUTES;
	pub const LaunchPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
	pub const PreimageByteDeposit: Balance = 1 * MICROUNIT;
	pub const VotingPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
	pub const MinimumDeposit: Balance = 100 * UNIT;
}

pub struct DummyOrigin {}
impl EnsureOrigin<Origin> for DummyOrigin {
	type Success = AccountId;

	fn try_origin(o: Origin) -> Result<Self::Success, Origin> {
		Into::<Result<RawOrigin<AccountId>, Origin>>::into(o).and_then(|o| match o {
			RawOrigin::Root => Ok(AccountId32::new([1;32])),
			r => Err(Origin::from(r)),
		})
	}
}

impl pallet_democracy::Config for Test {
	type Proposal = Call;
	type Event = Event;
	type Currency = Balances;
	type EnactmentPeriod = EnactmentPeriod;
	type LaunchPeriod = LaunchPeriod;
	type MinimumDeposit = MinimumDeposit;
	type PreimageByteDeposit = PreimageByteDeposit;
	type CooloffPeriod = VotingPeriod;
	type FastTrackVotingPeriod = VotingPeriod;
	type VoteLockingPeriod = VotingPeriod;
	type VotingPeriod = VotingPeriod;
	type MaxVotes = ConstU32<100>;
	type MaxProposals = ConstU32<100>;
	type ExternalOrigin = DummyOrigin;
	type ExternalMajorityOrigin = DummyOrigin;
	type ExternalDefaultOrigin = DummyOrigin;
	type FastTrackOrigin = DummyOrigin;
	type CancellationOrigin = DummyOrigin;
	type BlacklistOrigin = EnsureRoot<AccountId>;
	type CancelProposalOrigin = EnsureRoot<AccountId>;
	type VetoOrigin = DummyOrigin;
	type Slash = ();
	type InstantOrigin = DummyOrigin;
	type InstantAllowed = frame_support::traits::ConstBool<true>;
	type Scheduler = Scheduler;
	type OperationalPreimageOrigin = DummyOrigin;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
}

impl pallet_preimage::Config for Test {
	type Event = Event;
	type WeightInfo = ();
	type Currency = ();
	type ManagerOrigin = EnsureRoot<AccountId>;
	type MaxSize = ConstU32<1024>;
	type BaseDeposit = ();
	type ByteDeposit = ();
}

impl pallet_scheduler::Config for Test {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = ConstU64<2_000_000_000_000>;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type MaxScheduledPerBlock = ConstU32<100>;
	type WeightInfo = ();
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type PreimageProvider = Preimage;
	type NoPreimagePostponement = ConstU64<10>;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ConstU32<10>;
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU64<1_000_000_000>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
}

parameter_types! {
	pub const MTHoldersMotionDuration: BlockNumber = 1;
	pub const MTHoldersMaxProposals: u32 = 0;
	pub const MTHoldersMaxMembers: u32 = 2;
}

type MTHoldersInstance = pallet_collective::Instance1;
impl pallet_collective::Config<MTHoldersInstance> for Test {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = MTHoldersMotionDuration;
	type MaxProposals = MTHoldersMaxProposals;
	type MaxMembers = MTHoldersMaxMembers;
	type DefaultVote = ();
	type WeightInfo = ();
}

// I tried a mock democracy pallet but there were too many dependencies from the democracy pallet
// needed to get the test running.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext: sp_io::TestExternalities = GenesisConfig {
		mt_holders: pallet_collective::GenesisConfig {
			members: vec![1, 2],
			phantom: Default::default(),
		},
		default_collective: Default::default(),
	}
		.build_storage()
		.unwrap()
		.into();
	ext.execute_with(|| System::set_block_number(1));
	ext
}
//
// pub fn new_test_ext() -> sp_io::TestExternalities {
// 	let t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
// 	let mut ext = sp_io::TestExternalities::new(t);
// 	ext.execute_with(|| System::set_block_number(1));
// 	ext
// }
