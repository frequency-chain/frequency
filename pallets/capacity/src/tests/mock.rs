use crate as pallet_capacity;

use crate::{BalanceOf, StakingRewardClaim, StakingRewardsProvider};
use common_primitives::{
	node::{AccountId, Hash, Header, ProposalProvider},
	schema::{SchemaId, SchemaValidator},
};
use frame_support::{
	parameter_types,
	traits::{ConstU16, ConstU32, ConstU64},
};
use frame_system::EnsureSigned;
use sp_core::{ConstU8, H256};
use sp_runtime::{
	traits::{BlakeTwo256, Convert, IdentityLookup},
	AccountId32, DispatchError, Perbill,
};
use sp_std::ops::{Mul, Div};

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
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Msa: pallet_msa::{Pallet, Call, Storage, Event<T>},
		Capacity: pallet_capacity::{Pallet, Call, Storage, Event<T>},
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
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_balances::Config for Test {
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type MaxLocks = ConstU32<10>;
	type Balance = u64;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type FreezeIdentifier = ();
	type HoldIdentifier = ();
	type MaxFreezes = ConstU32<0>;
	type MaxHolds = ConstU32<0>;
}

pub type MaxSchemaGrantsPerDelegation = ConstU32<30>;

pub struct TestAccountId;

impl Convert<u64, AccountId> for TestAccountId {
	fn convert(_x: u64) -> AccountId32 {
		AccountId32::new([1u8; 32])
	}
}
pub struct Schemas;
impl SchemaValidator<SchemaId> for Schemas {
	fn are_all_schema_ids_valid(_schema_id: &Vec<SchemaId>) -> bool {
		true
	}

	fn set_schema_count(_n: SchemaId) {}
}
pub struct CouncilProposalProvider;

impl ProposalProvider<u64, RuntimeCall> for CouncilProposalProvider {
	fn propose(
		_who: u64,
		_threshold: u32,
		_proposal: Box<RuntimeCall>,
	) -> Result<(u32, u32), DispatchError> {
		Ok((1u32, 1u32))
	}

	fn propose_with_simple_majority(
		_who: u64,
		_proposal: Box<RuntimeCall>,
	) -> Result<(u32, u32), DispatchError> {
		Ok((1u32, 1u32))
	}

	#[cfg(any(feature = "runtime-benchmarks", feature = "test"))]
	fn proposal_count() -> u32 {
		1u32
	}
}

impl pallet_msa::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type ConvertIntoAccountId32 = TestAccountId;
	type MaxPublicKeysPerMsa = ConstU8<255>;
	type MaxSchemaGrantsPerDelegation = MaxSchemaGrantsPerDelegation;
	type MaxProviderNameSize = ConstU32<16>;
	type SchemaValidator = Schemas;
	type HandleProvider = ();
	type MortalityWindowSize = ConstU32<100>;
	type Proposal = RuntimeCall;
	type ProposalProvider = CouncilProposalProvider;
	type CreateProviderViaGovernanceOrigin = EnsureSigned<u64>;
	/// This MUST ALWAYS be MaxSignaturesPerBucket * NumberOfBuckets.
	type MaxSignaturesStored = ConstU32<8000>;
}

// not used yet
pub struct TestStakingRewardsProvider {}

type TestRewardEra = u32;

impl StakingRewardsProvider<Test> for TestStakingRewardsProvider {
	type AccountId = u64;
	type RewardEra = TestRewardEra;
	type Hash = Hash; // use what's in common_primitives::node

	fn reward_pool_size(total_staked: BalanceOf<Test>) -> BalanceOf<Test> {
		total_staked.div(10u64)
	}

	fn staking_reward_total(
		account_id: Self::AccountId,
		_from_era: Self::RewardEra,
		_to_era: Self::RewardEra,
	) -> Result<BalanceOf<Test>, DispatchError> {
		if account_id > 2u64 {
			Ok(10u64)
		} else {
			Ok(1u64)
		}
	}

	fn validate_staking_reward_claim(
		_account_id: Self::AccountId,
		_proof: Self::Hash,
		_payload: StakingRewardClaim<Test>,
	) -> bool {
		true
	}

	fn capacity_boost(amount: BalanceOf<Test>) -> BalanceOf<Test> {
		Perbill::from_percent(5u32).mul(amount)
	}
}

// Needs parameter_types! for the Perbill
parameter_types! {
	pub const TestCapacityPerToken: Perbill = Perbill::from_percent(10);
}
impl pallet_capacity::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type Currency = pallet_balances::Pallet<Self>;
	type TargetValidator = Msa;
	// In test, this must be >= Token:Capacity ratio since unit is plancks
	type MinimumStakingAmount = ConstU64<10>;
	type MinimumTokenBalance = ConstU64<10>;
	type MaxUnlockingChunks = ConstU32<4>;

	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = Msa;

	type UnstakingThawPeriod = ConstU16<2>;
	type MaxEpochLength = ConstU32<100>;
	type EpochNumber = u32;
	type CapacityPerToken = TestCapacityPerToken;
	type RewardEra = TestRewardEra;
	type EraLength = ConstU32<10>;
	type StakingRewardsPastErasMax = ConstU32<5>;
	type RewardsProvider = TestStakingRewardsProvider;
	type ChangeStakingTargetThawEras = ConstU32<5>;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![
			(50, 5),
			(100, 100),
			(200, 200),
			(300, 300),
			(400, 400),
			(500, 500),
			(600, 600),
			(10_000, 10_000),
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
