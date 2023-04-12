use super::*;
use crate as pallet_frequency_tx_payment;

use common_primitives::{
	handles::{HandleProvider, HandleResponse},
	msa::MessageSourceId,
	node::{AccountId, ProposalProvider},
	schema::{SchemaId, SchemaValidator},
};
use frame_system::EnsureSigned;
use pallet_transaction_payment::CurrencyAdapter;
use sp_core::{ConstU8, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, Convert, IdentityLookup, SaturatedConversion},
	AccountId32,
};

use frame_support::{
	parameter_types,
	traits::{ConstU16, ConstU64},
	weights::WeightToFee as WeightToFeeTrait,
};

use frame_support::weights::Weight;

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
			TransactionPayment: pallet_transaction_payment::{Pallet, Storage, Event<T>},
			FrequencyTxPayment: pallet_frequency_tx_payment::{Pallet, Call, Event<T>},
			Utility: pallet_utility::{Pallet, Call, Storage, Event},
		}
);

pub struct BlockWeights;
impl Get<frame_system::limits::BlockWeights> for BlockWeights {
	fn get() -> frame_system::limits::BlockWeights {
		frame_system::limits::BlockWeights::builder()
			.base_block(Weight::zero())
			.for_class(DispatchClass::all(), |weights| {
				weights.base_extrinsic = ExtrinsicBaseWeight::get().into();
			})
			.for_class(DispatchClass::non_mandatory(), |weights| {
				weights.max_total = Weight::from_ref_time(1024).set_proof_size(u64::MAX).into();
			})
			.build_or_panic()
	}
}

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = BlockWeights;
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
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_balances::Config for Test {
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type Balance = u64;
	type MaxLocks = ();
	type WeightInfo = ();
	type ReserveIdentifier = [u8; 8];
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type MaxReserves = ();
}

parameter_types! {
	pub const MaxSchemaGrantsPerDelegation: u32 = 30;
	pub const MaximumCapacityBatchLength: u8 = 2;
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

pub struct MockHandleProvider;
impl HandleProvider for MockHandleProvider {
	fn get_handle_for_msa(msa_id: MessageSourceId) -> Option<HandleResponse> {
		if msa_id == 1 {
			Some(HandleResponse {
				base_handle: "test1".into(),
				canonical_handle: "test1".into(),
				suffix: 2u16,
			})
		} else {
			None
		}
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
	type HandleProvider = MockHandleProvider;
	type MortalityWindowSize = ConstU32<100>;
	type Proposal = RuntimeCall;
	type ProposalProvider = CouncilProposalProvider;
	type CreateProviderViaGovernanceOrigin = EnsureSigned<u64>;
	/// This MUST ALWAYS be MaxSignaturesPerBucket * NumberOfBuckets.
	type MaxSignaturesStored = ConstU32<8000>;
}

parameter_types! {
	pub static WeightToFee: u64 = 1;
	pub static TransactionByteFee: u64 = 1;
	static ExtrinsicBaseWeight: Weight = Weight::zero();
}

impl WeightToFeeTrait for WeightToFee {
	type Balance = u64;

	fn weight_to_fee(weight: &Weight) -> Self::Balance {
		Self::Balance::saturated_from(weight.ref_time())
			.saturating_mul(WEIGHT_TO_FEE.with(|v| *v.borrow()))
	}
}

impl WeightToFeeTrait for TransactionByteFee {
	type Balance = u64;

	fn weight_to_fee(weight: &Weight) -> Self::Balance {
		Self::Balance::saturated_from(weight.ref_time())
			.saturating_mul(TRANSACTION_BYTE_FEE.with(|v| *v.borrow()))
	}
}

impl pallet_transaction_payment::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = CurrencyAdapter<Balances, ()>;
	type WeightToFee = WeightToFee;
	type LengthToFee = TransactionByteFee;
	type FeeMultiplierUpdate = ();
	type OperationalFeeMultiplier = ConstU8<5>;
}

impl pallet_capacity::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type Currency = pallet_balances::Pallet<Self>;
	type TargetValidator = ();
	type MinimumStakingAmount = ConstU64<5>;
	type MinimumTokenBalance = ConstU64<10>;
	type MaxUnlockingChunks = ConstU32<4>;

	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();

	type UnstakingThawPeriod = ConstU16<2>;
	type MaxEpochLength = ConstU64<100>;
	type EpochNumber = u32;
}

use pallet_balances::Call as BalancesCall;

pub struct TestCapacityCalls;

impl GetStableWeight<RuntimeCall, Weight> for TestCapacityCalls {
	fn get_stable_weight(call: &RuntimeCall) -> Option<Weight> {
		match call {
			RuntimeCall::Balances(BalancesCall::transfer { .. }) => Some(Weight::from_ref_time(11)),
			_ => None,
		}
	}
}

pub struct CapacityBatchProvider;

impl UtilityProvider<RuntimeOrigin, RuntimeCall> for CapacityBatchProvider {
	fn batch_all(origin: RuntimeOrigin, calls: Vec<RuntimeCall>) -> DispatchResultWithPostInfo {
		Utility::batch_all(origin, calls)
	}
}

impl pallet_utility::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
}

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Capacity = Capacity;
	type WeightInfo = ();
	type CapacityCalls = TestCapacityCalls;
	type OnChargeCapacityTransaction = payment::CapacityAdapter<Balances, Msa>;
	type MaximumCapacityBatchLength = MaximumCapacityBatchLength;
	type BatchProvider = CapacityBatchProvider;
}

pub struct ExtBuilder {
	balance_factor: u64,
	base_weight: Weight,
	byte_fee: u64,
	weight_to_fee: u64,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			balance_factor: 1,
			base_weight: Weight::from_ref_time(0),
			byte_fee: 1,
			weight_to_fee: 1,
		}
	}
}

impl ExtBuilder {
	fn set_constants(&self) {
		ExtrinsicBaseWeight::mutate(|v| *v = self.base_weight);
		TRANSACTION_BYTE_FEE.with(|v| *v.borrow_mut() = self.byte_fee);
		WEIGHT_TO_FEE.with(|v| *v.borrow_mut() = self.weight_to_fee);
	}

	pub fn base_weight(mut self, base_weight: Weight) -> Self {
		self.base_weight = base_weight;
		self
	}

	pub fn balance_factor(mut self, factor: u64) -> Self {
		self.balance_factor = factor;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		self.set_constants();

		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
		pallet_balances::GenesisConfig::<Test> {
			balances: if self.balance_factor > 0 {
				vec![
					(1, 10 * self.balance_factor),
					(2, 20 * self.balance_factor),
					(3, 30 * self.balance_factor),
					(4, 40 * self.balance_factor),
					(5, 50 * self.balance_factor),
					(6, 60 * self.balance_factor),
				]
			} else {
				vec![]
			},
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut t: sp_io::TestExternalities = t.into();

		// Create MSA account 1 - 6 and add Balance to them with Capacity balance
		t.execute_with(|| {
			let msa_accounts: Vec<(
				<Test as frame_system::Config>::AccountId,
				<Test as pallet_balances::Config>::Balance,
			)> = vec![
				(1, 10 * self.balance_factor),
				(2, 20 * self.balance_factor),
				(3, 30 * self.balance_factor),
				(4, 40 * self.balance_factor),
				(5, 50 * self.balance_factor),
				(6, 60 * self.balance_factor),
			];
			msa_accounts.iter().for_each(|(account, balance)| {
				let msa_id = create_msa_account(account.clone());
				create_capacity_for(msa_id, balance.clone());
			});
		});

		t.into()
	}
}

pub fn create_msa_account(
	account_id: <Test as frame_system::Config>::AccountId,
) -> MessageSourceId {
	pub const EMPTY_FUNCTION: fn(MessageSourceId) -> DispatchResult = |_| Ok(());
	let (msa_id, _) = Msa::create_account(account_id, EMPTY_FUNCTION).unwrap();

	msa_id
}

fn create_capacity_for(target: MessageSourceId, amount: u64) {
	let mut capacity_details = Capacity::get_capacity_for(target).unwrap_or_default();
	capacity_details.deposit(&amount).unwrap();
	Capacity::set_capacity_for(target, capacity_details);
}
