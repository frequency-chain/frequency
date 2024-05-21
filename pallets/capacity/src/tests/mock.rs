use crate as pallet_capacity;

use crate::{
	tests::testing_utils::set_era_and_reward_pool, BalanceOf, StakingRewardClaim,
	StakingRewardsProvider,
};
use common_primitives::{
	node::{AccountId, Hash, ProposalProvider},
	schema::{SchemaId, SchemaValidator},
};
use frame_support::{
	construct_runtime, parameter_types,
	traits::{ConstU16, ConstU32, ConstU64},
};
use frame_system::EnsureSigned;
use sp_core::{ConstU8, H256};
use sp_runtime::{
	traits::{BlakeTwo256, Convert, IdentityLookup},
	AccountId32, BuildStorage, DispatchError, Perbill, Permill,
};
use sp_std::ops::Mul;

type Block = frame_system::mocking::MockBlockU32<Test>;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Test
	{
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Msa: pallet_msa::{Pallet, Call, Storage, Event<T>},
		Capacity: pallet_capacity::{Pallet, Call, Storage, Event<T>, FreezeReason},
	}
);

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Block = Block;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
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
	type FreezeIdentifier = RuntimeFreezeReason;
	type MaxFreezes = ConstU32<1>;
	type MaxHolds = ConstU32<0>;
	type RuntimeHoldReason = ();
	type RuntimeFreezeReason = ();
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
	type Balance = BalanceOf<Test>;

	// To reflect new economic model behavior of having a constant RewardPool amount.
	fn reward_pool_size(_total_staked: Self::Balance) -> Self::Balance {
		10_000u64.into()
	}

	fn staking_reward_total(
		account_id: Self::AccountId,
		_from_era: Self::RewardEra,
		_to_era: Self::RewardEra,
	) -> Result<Self::Balance, DispatchError> {
		if account_id > 2u64 {
			Ok(10u64)
		} else {
			Ok(1u64)
		}
	}

	// use the pallet version of the era calculation.
	fn era_staking_reward(
		amount_staked: Self::Balance,
		total_staked: Self::Balance,
		reward_pool_size: Self::Balance,
	) -> Self::Balance {
		Capacity::era_staking_reward(amount_staked, total_staked, reward_pool_size)
	}

	fn validate_staking_reward_claim(
		_account_id: Self::AccountId,
		_proof: Self::Hash,
		_payload: StakingRewardClaim<Test>,
	) -> bool {
		true
	}

	fn capacity_boost(amount: Self::Balance) -> Self::Balance {
		Perbill::from_percent(50u32).mul(amount)
	}
}

// Needs parameter_types! for the Perbill
parameter_types! {
	pub const TestCapacityPerToken: Perbill = Perbill::from_percent(10);
	pub const TestRewardCap: Permill = Permill::from_parts(3_800); // 0.38% or 0.0038 per RewardEra
}
impl pallet_capacity::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type RuntimeFreezeReason = RuntimeFreezeReason;
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
	type StakingRewardsPastErasMax = ConstU32<6>; // 5 for claiming rewards, 1 for current reward era
	type RewardsProvider = Capacity;
	type MaxRetargetsPerRewardEra = ConstU32<5>;
	type RewardPoolEachEra = ConstU64<10_000>;
	type RewardPercentCap = TestRewardCap;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
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
	ext.execute_with(|| {
		System::set_block_number(1);
		set_era_and_reward_pool(1, 1, 0);
	});
	ext
}
