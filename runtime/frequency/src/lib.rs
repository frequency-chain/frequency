#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

// Don't allow both frequency and all-frequency-features so that we always have a good mainnet runtime
#[cfg(all(feature = "frequency", feature = "all-frequency-features"))]
compile_error!("feature \"frequency\" and feature \"all-frequency-features\" cannot be enabled at the same time");

mod benchmarking;

use cumulus_pallet_parachain_system::{
	RelayNumberStrictlyIncreases, RelaychainBlockNumberProvider,
};
use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{AccountIdConversion, AccountIdLookup, BlakeTwo256, Block as BlockT, ConvertInto},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult,
};

#[cfg(feature = "runtime-benchmarks")]
use codec::Decode;

use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

use common_primitives::{
	messages::*,
	msa::*,
	node::*,
	schema::{PayloadLocation, SchemaResponse},
};

pub use common_runtime::{
	constants::{currency::EXISTENTIAL_DEPOSIT, *},
	fee::WeightToFee,
};

use frame_support::{
	construct_runtime,
	dispatch::DispatchClass,
	parameter_types,
	traits::{ConstU128, ConstU32, EitherOfDiverse, EnsureOrigin, EqualPrivilegeOnly},
	weights::{constants::RocksDbWeight, ConstantMultiplier, Weight},
};

use frame_system::{
	limits::{BlockLength, BlockWeights},
	EnsureRoot, RawOrigin,
};
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
pub use sp_runtime::{MultiAddress, Perbill, Permill};

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

pub use pallet_msa;
pub use pallet_schemas;
// Polkadot Imports
use polkadot_runtime_common::{BlockHashCount, SlowAdjustingFeeUpdate};

pub use common_runtime::{
	constants::MaxDataSize,
	weights,
	weights::{BlockExecutionWeight, ExtrinsicBaseWeight},
};
use frame_support::traits::Contains;

/// Basefilter to only allow specified transactions call to be executed
/// For non mainnet [--features frequency] all transactions are allowed
pub struct BaseCallFilter;

impl Contains<RuntimeCall> for BaseCallFilter {
	fn contains(_call: &RuntimeCall) -> bool {
		#[cfg(not(feature = "frequency"))]
		{
			true
		}
		#[cfg(feature = "frequency")]
		{
			matches!(
				_call,
				RuntimeCall::System(..) |
					RuntimeCall::Timestamp(..) |
					RuntimeCall::ParachainSystem(..) |
					RuntimeCall::TechnicalCommittee(..) |
					RuntimeCall::Council(..) |
					RuntimeCall::Democracy(..) |
					RuntimeCall::Session(..) |
					RuntimeCall::Preimage(..) |
					RuntimeCall::Scheduler(..)
			)
		}
	}
}

/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	common_runtime::extensions::check_nonce::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
	pallet_msa::CheckFreeExtrinsicUse<Runtime>,
);
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;

/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;

/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, RuntimeCall, SignedExtra>;

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
>;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	use super::*;
	use sp_runtime::{generic, traits::BlakeTwo256};

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;
}

impl_opaque_keys! {
	pub struct SessionKeys {
		pub aura: Aura,
	}
}

// The duplicate macros are annoying, but #[sp_version::runtime_version]
// has fairly string limits on what can go in there.

// Override the spec name when not mainnet to be frequency-rococo
#[cfg(not(feature = "frequency"))]
macro_rules! spec_name {
	( $y:expr ) => {{
		create_runtime_str!("frequency-rococo")
	}};
}

#[cfg(feature = "frequency")]
macro_rules! spec_name {
	( $y:expr ) => {{
		create_runtime_str!($y)
	}};
}

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: spec_name!("frequency"),
	impl_name: create_runtime_str!("frequency"),
	authoring_version: 1,
	spec_version: 9,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;

	// This part is copied from Substrate's `bin/node/runtime/src/lib.rs`.
	//  The `RuntimeBlockLength` and `RuntimeBlockWeights` exist here because the
	// `DeletionWeightLimit` and `DeletionQueueDepth` depend on those to parameterize
	// the lazy contract deletion.
	pub RuntimeBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);

	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// Base call filter to use in dispatchable.
	// enable for cfg feature "frequency" only
	type BaseCallFilter = BaseCallFilter;
	/// The aggregated dispatch type that is available for extrinsics.
	type RuntimeCall = RuntimeCall;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = AccountIdLookup<AccountId, ()>;
	/// The index type for storing how many extrinsics an account has signed.
	type Index = Index;
	/// The index type for blocks.
	type BlockNumber = BlockNumber;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The header type.
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	/// The ubiquitous origin type.
	type RuntimeOrigin = RuntimeOrigin;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// Runtime version.
	type Version = Version;
	/// Converts a module to an index of this module in the runtime.
	type PalletInfo = PalletInfo;
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = ();
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = ();
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = RuntimeBlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = RuntimeBlockLength;
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = Ss58Prefix;
	/// The action to take on a Runtime Upgrade
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	type MaxConsumers = FrameSystemMaxConsumers;
}

impl pallet_msa::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_msa::weights::SubstrateWeight<Runtime>;
	// The conversion to a 32 byte AccountId
	type ConvertIntoAccountId32 = ConvertInto;
	// The maximum number of public keys per MSA
	type MaxPublicKeysPerMsa = MsaMaxPublicKeysPerMsa;
	// The maximum number of schema grants per delegation
	type MaxSchemaGrantsPerDelegation = MaxDataSize;
	// The maximum provider name size (in bytes)
	type MaxProviderNameSize = MsaMaxProviderNameSize;
	// The type that provides schema related info
	type SchemaValidator = Schemas;
	// The number of blocks per virtual bucket
	type MortalityWindowSize = MSAMortalityWindowSize;
	// The maximum number of signatures per virtual bucket
	type MaxSignaturesPerBucket = MSAMaxSignaturesPerBucket;
	// The total number of virtual buckets
	type NumberOfBuckets = MSANumberOfBuckets;
	// The maximum number of signatures that can be stored in the payload signature registry
	type MaxSignaturesStored = MSAMaxSignaturesStored;
}

pub use common_primitives::schema::SchemaId;

impl pallet_schemas::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_schemas::weights::SubstrateWeight<Runtime>;
	// The mininum size (in bytes) for a schema model
	type MinSchemaModelSizeBytes = SchemasMinModelSizeBytes;
	// The maximum number of schemas that can be registered
	type MaxSchemaRegistrations = SchemasMaxRegistrations;
	// The maximum length of a schema model (in bytes)
	type SchemaModelMaxBytesBoundedVecLimit = SchemasMaxBytesBoundedVecLimit;
}

pub struct RootAsVestingPallet;
impl EnsureOrigin<RuntimeOrigin> for RootAsVestingPallet {
	type Success = AccountId;

	fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
		Into::<Result<RawOrigin<AccountId>, RuntimeOrigin>>::into(o).and_then(|o| match o {
			RawOrigin::Root => Ok(VestingPalletId::get().into_account_truncating()),
			r => Err(RuntimeOrigin::from(r)),
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_origin() -> RuntimeOrigin {
		let zero_account_id =
			AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
				.expect("infinite length input; no invalid inputs for type; qed");
		RuntimeOrigin::from(RawOrigin::Signed(zero_account_id))
	}
}

parameter_types! {
	/// Need this declaration method for use + type safety in benchmarks
	pub const MaxVestingSchedules: u32 = ORML_MAX_VESTING_SCHEDULES;
}

// See https://paritytech.github.io/substrate/master/pallet_vesting/index.html for
// the descriptions of these configs.
impl orml_vesting::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type MinVestedTransfer = MinVestedTransfer;
	type VestedTransferOrigin = RootAsVestingPallet;
	type WeightInfo = weights::orml_vesting::SubstrateWeight<Runtime>;
	type MaxVestingSchedules = MaxVestingSchedules;
	type BlockNumberProvider = RelaychainBlockNumberProvider<Runtime>;
}

// See https://paritytech.github.io/substrate/master/pallet_timestamp/index.html for
// the descriptions of these configs.
impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = weights::pallet_timestamp::SubstrateWeight<Runtime>;
}

// See https://paritytech.github.io/substrate/master/pallet_authorship/index.html for
// the descriptions of these configs.
impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type UncleGenerations = AuthorshipUncleGenerations;
	type FilterUncle = ();
	type EventHandler = (CollatorSelection,);
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = BalancesMaxLocks;
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
	type AccountStore = System;
	type WeightInfo = weights::pallet_balances::SubstrateWeight<Runtime>;
	type MaxReserves = BalancesMaxReserves;
	type ReserveIdentifier = [u8; 8];
}
parameter_types! {
	// The maximum weight that may be scheduled per block for any dispatchables of less priority than schedule::HARD_DEADLINE.
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(10) * RuntimeBlockWeights::get().max_block;
}

// See also https://docs.rs/pallet-scheduler/latest/pallet_scheduler/trait.Config.html
impl pallet_scheduler::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type MaximumWeight = MaximumSchedulerWeight;
	/// Origin to schedule or cancel calls
	/// Set to Root or a simple majority of the Frequency Council
	type ScheduleOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 2>,
	>;
	type MaxScheduledPerBlock = SchedulerMaxScheduledPerBlock;
	type WeightInfo = weights::pallet_scheduler::SubstrateWeight<Runtime>;
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type Preimages = Preimage;
}

// See https://paritytech.github.io/substrate/master/pallet_preimage/index.html for
// the descriptions of these configs.
impl pallet_preimage::Config for Runtime {
	type WeightInfo = weights::pallet_preimage::SubstrateWeight<Runtime>;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	// Allow the Technical council to request preimages without deposit or fees
	type ManagerOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureMember<AccountId, TechnicalCommitteeCollective>,
	>;
	type BaseDeposit = PreimageBaseDeposit;
	type ByteDeposit = PreimageByteDeposit;
}

// See https://paritytech.github.io/substrate/master/pallet_collective/index.html for
// the descriptions of these configs.
type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

type TechnicalCommitteeCollective = pallet_collective::Instance2;
impl pallet_collective::Config<TechnicalCommitteeCollective> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = TCMotionDuration;
	type MaxProposals = TCMaxProposals;
	type MaxMembers = TCMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	// TODO: this uses default but we don't have weights yet. Issue: #608
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

// see https://paritytech.github.io/substrate/master/pallet_democracy/pallet/trait.Config.html
// for the definitions of these configs
impl pallet_democracy::Config for Runtime {
	type CooloffPeriod = CooloffPeriod;
	type Currency = Balances;
	type EnactmentPeriod = EnactmentPeriod;
	type RuntimeEvent = RuntimeEvent;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	type InstantAllowed = frame_support::traits::ConstBool<true>;
	type LaunchPeriod = LaunchPeriod;
	type MaxProposals = DemocracyMaxProposals;
	type MaxVotes = DemocracyMaxVotes;
	type MinimumDeposit = MinimumDeposit;
	type Scheduler = Scheduler;
	type Slash = (); // Treasury;
	type WeightInfo = weights::pallet_democracy::SubstrateWeight<Runtime>;
	type VoteLockingPeriod = EnactmentPeriod; // Same as EnactmentPeriod
	type VotingPeriod = VotingPeriod;
	type Preimages = Preimage;
	type MaxDeposits = ConstU32<100>;
	type MaxBlacklisted = ConstU32<100>;

	// See https://paritytech.github.io/substrate/master/pallet_democracy/index.html for
	// the descriptions of these origins.
	/// A unanimous council can have the next scheduled referendum be a straight default-carries
	/// (NTB) vote.
	type ExternalDefaultOrigin = EitherOfDiverse<
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 1>,
		frame_system::EnsureRoot<AccountId>,
	>;

	/// A super-majority of 3/5ths can have the next scheduled referendum be a straight majority-carries vote.
	type ExternalMajorityOrigin = EitherOfDiverse<
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 5>,
		frame_system::EnsureRoot<AccountId>,
	>;
	/// A straight majority (at least 50%) of the council can decide what their next motion is.
	type ExternalOrigin = EitherOfDiverse<
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 2>,
		frame_system::EnsureRoot<AccountId>,
	>;
	/// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
	/// be tabled immediately and with a shorter voting/enactment period.
	type FastTrackOrigin = EitherOfDiverse<
		pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCommitteeCollective, 2, 3>,
		frame_system::EnsureRoot<AccountId>,
	>;
	/// Origin from which the next majority-carries (or more permissive) referendum may be tabled to
	/// vote immediately and asynchronously in a similar manner to the emergency origin.
	/// Requires TechnicalCommittee to be unanimous.
	type InstantOrigin = EitherOfDiverse<
		pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCommitteeCollective, 1, 1>,
		frame_system::EnsureRoot<AccountId>,
	>;
	/// Overarching type of all pallets origins
	type PalletsOrigin = OriginCaller;

	/// To cancel a proposal which has been passed, 2/3 of the council must agree to it.
	type CancellationOrigin = EitherOfDiverse<
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 2, 3>,
		EnsureRoot<AccountId>,
	>;
	/// To cancel a proposal before it has been passed, the technical committee must be unanimous or
	/// Root must agree.
	type CancelProposalOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCommitteeCollective, 1, 1>,
	>;

	/// This origin can blacklist proposals.
	type BlacklistOrigin = EnsureRoot<AccountId>;

	/// Any single technical committee member may veto a coming council proposal, however they can
	/// only do it once and it lasts only for the cool-off period.
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCommitteeCollective>;
}

// See https://paritytech.github.io/substrate/master/pallet_treasury/index.html for
// the descriptions of these configs.
impl pallet_treasury::Config for Runtime {
	/// Treasury Account: 5EYCAe5ijiYfyeZ2JJCGq56LmPyNRAKzpG4QkoQkkQNB5e6Z
	type PalletId = TreasuryPalletId;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = weights::pallet_treasury::SubstrateWeight<Runtime>;

	/// Who approves treasury proposals?
	/// - Root (sudo or governance)
	/// - 3/5ths of the Frequency Council
	type ApproveOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 5>,
	>;

	/// Who rejects treasury proposals?
	/// - Root (sudo or governance)
	/// - Simple majority of the Frequency Council
	type RejectOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>,
	>;

	/// Spending funds outside of the proposal?
	/// Nobody
	type SpendOrigin = frame_support::traits::NeverEnsureOrigin<Balance>;

	/// Rejected proposals lose their bond
	/// This takes the slashed amount and is often set to the Treasury
	/// We burn it so there is no incentive to the treasury to reject to enrich itself
	type OnSlash = ();

	/// Bond 5% of a treasury proposal
	type ProposalBond = ProposalBondPercent;

	/// Minimum bond of 100 Tokens
	type ProposalBondMinimum = ProposalBondMinimum;

	/// Max bond of 1_000 Tokens
	type ProposalBondMaximum = ProposalBondMaximum;

	/// Pay out on a 4-week basis
	type SpendPeriod = SpendPeriod;

	/// Do not burn any unused funds
	type Burn = ();

	/// Where should tokens burned from the treasury go?
	/// Set to go to /dev/null
	type BurnDestination = ();

	/// Runtime hooks to external pallet using treasury to compute spend funds.
	/// Set to Bounties often.
	/// Not currently in use
	type SpendFunds = ();

	/// 64
	type MaxApprovals = MaxApprovals;
}

// See https://paritytech.github.io/substrate/master/pallet_transaction_payment/index.html for
// the descriptions of these configs.
impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
	type WeightToFee = WeightToFee;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
	type OperationalFeeMultiplier = TransactionPaymentOperationalFeeMultiplier;
}

// See https://paritytech.github.io/substrate/master/pallet_parachain_system/index.html for
// the descriptions of these configs.
impl cumulus_pallet_parachain_system::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnSystemEvent = ();
	type SelfParaId = parachain_info::Pallet<Runtime>;
	type DmpMessageHandler = ();
	type ReservedDmpWeight = ();
	type OutboundXcmpMessageSource = ();
	type XcmpMessageHandler = ();
	type ReservedXcmpWeight = ();
	type CheckAssociatedRelayNumber = RelayNumberStrictlyIncreases;
}

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_aura_ext::Config for Runtime {}

// See https://paritytech.github.io/substrate/master/pallet_session/index.html for
// the descriptions of these configs.
impl pallet_session::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	// we don't have stash and controller, thus we don't need the convert as well.
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ShouldEndSession = pallet_session::PeriodicSessions<SessionPeriod, SessionOffset>;
	type NextSessionRotation = pallet_session::PeriodicSessions<SessionPeriod, SessionOffset>;
	type SessionManager = CollatorSelection;
	// Essentially just Aura, but lets be pedantic.
	type SessionHandler = <SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	type WeightInfo = weights::pallet_session::SubstrateWeight<Runtime>;
}

// See https://paritytech.github.io/substrate/master/pallet_aura/index.html for
// the descriptions of these configs.
impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = AuraMaxAuthorities;
}

// See https://paritytech.github.io/substrate/master/pallet_collator_selection/index.html for
// the descriptions of these configs.
impl pallet_collator_selection::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;

	// Origin that can dictate updating parameters of this pallet.
	// Currently only root or a 3/5ths council vote.
	type UpdateOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 5>,
	>;

	// Account Identifier from which the internal Pot is generated.
	// Set to something that NEVER gets a balance i.e. No block rewards.
	type PotId = NeverDepositIntoId;

	// Maximum number of candidates that we should have. This is enforced in code.
	//
	// This does not take into account the invulnerables.
	type MaxCandidates = CollatorMaxCandidates;

	// Minimum number of candidates that we should have. This is used for disaster recovery.
	//
	// This does not take into account the invulnerables.
	type MinCandidates = CollatorMinCandidates;

	// Maximum number of invulnerables. This is enforced in code.
	type MaxInvulnerables = CollatorMaxInvulnerables;

	// Will be kicked if block is not produced in threshold.
	// should be a multiple of session or things will get inconsistent
	type KickThreshold = CollatorKickThreshold;

	/// A stable ID for a validator.
	type ValidatorId = <Self as frame_system::Config>::AccountId;

	// A conversion from account ID to validator ID.
	//
	// Its cost must be at most one storage read.
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;

	// Validate a user is registered
	type ValidatorRegistration = Session;

	type WeightInfo = ();
}

impl pallet_messages::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_messages::weights::SubstrateWeight<Runtime>;
	// The type that supplies MSA info
	type MsaInfoProvider = Msa;
	// The type that validates schema grants
	type SchemaGrantValidator = Msa;
	// The type that provides schema info
	type SchemaProvider = Schemas;
	// The maximum number of messages per block
	type MaxMessagesPerBlock = MessagesMaxPerBlock;
	// The maximum message payload in bytes
	type MaxMessagePayloadSizeBytes = MessagesMaxPayloadSizeBytes;

	/// A set of helper functions for benchmarking.
	#[cfg(feature = "runtime-benchmarks")]
	type MsaBenchmarkHelper = Msa;
	#[cfg(feature = "runtime-benchmarks")]
	type SchemaBenchmarkHelper = Schemas;
}

// See https://paritytech.github.io/substrate/master/pallet_sudo/index.html for
// the descriptions of these configs.
#[cfg(any(not(feature = "frequency"), feature = "all-frequency-features"))]
impl pallet_sudo::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
}

// See https://paritytech.github.io/substrate/master/pallet_utility/index.html for
// the descriptions of these configs.
impl pallet_utility::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = weights::pallet_utility::SubstrateWeight<Runtime>;
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		// System support stuff.
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>} = 0,
		ParachainSystem: cumulus_pallet_parachain_system::{
			Pallet, Call, Config, Storage, Inherent, Event<T>, ValidateUnsigned,
		} = 1,
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 2,
		ParachainInfo: parachain_info::{Pallet, Storage, Config} = 3,

		// Sudo removed from mainnet Jan 2023
		#[cfg(any(not(feature = "frequency"), feature = "all-frequency-features"))]
		Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T> }= 4,

		Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>} = 5,
		Democracy: pallet_democracy::{Pallet, Call, Config<T>, Storage, Event<T> } = 6,
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T> } = 8,
		Utility: pallet_utility::{Pallet, Call, Event} = 9,

		// Monetary stuff.
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 10,
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage, Event<T>} = 11,

		// Collectives
		Council: pallet_collective::<Instance1>::{Pallet, Call, Config<T,I>, Storage, Event<T>, Origin<T>} = 12,
		TechnicalCommittee: pallet_collective::<Instance2>::{Pallet, Call, Config<T,I>, Storage, Event<T>, Origin<T>} = 13,

		// Treasury
		Treasury: pallet_treasury::{Pallet, Call, Storage, Config, Event<T>} = 14,

		// Collator support. The order of these 4 are important and shall not change.
		Authorship: pallet_authorship::{Pallet, Call, Storage} = 20,
		CollatorSelection: pallet_collator_selection::{Pallet, Call, Storage, Event<T>, Config<T>} = 21,
		Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 22,
		Aura: pallet_aura::{Pallet, Storage, Config<T>} = 23,
		AuraExt: cumulus_pallet_aura_ext::{Pallet, Storage, Config} = 24,

		// ORML
		Vesting: orml_vesting::{Pallet, Call, Storage, Event<T>, Config<T>} = 40,

		// Frequency related pallets
		Msa: pallet_msa::{Pallet, Call, Storage, Event<T>} = 60,
		Messages: pallet_messages::{Pallet, Call, Storage, Event<T>} = 61,
		Schemas: pallet_schemas::{Pallet, Call, Storage, Event<T>, Config} = 62,
	}
);

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	define_benchmarks!(
		// Substrate
		[frame_system, SystemBench::<Runtime>]
		[pallet_balances, Balances]
		[pallet_collective, Council]
		[pallet_collective, TechnicalCommittee]
		[pallet_preimage, Preimage]
		[pallet_democracy, Democracy]
		[pallet_treasury, Treasury]
		[pallet_scheduler, Scheduler]
		[pallet_session, SessionBench::<Runtime>]
		[pallet_timestamp, Timestamp]
		[pallet_collator_selection, CollatorSelection]
		[pallet_utility, Utility]

		// Frequency
		[pallet_msa, Msa]
		[pallet_schemas, Schemas]
		[pallet_messages, Messages]
	);
}

impl_runtime_apis! {
	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities().into_inner()
		}
	}

	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
		fn account_nonce(account: AccountId) -> Index {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
	}

	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info(header: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info(header)
		}
	}

	// Frequency runtime APIs
	impl pallet_messages_runtime_api::MessagesRuntimeApi<Block> for Runtime {
		fn get_messages_by_schema_and_block(schema_id: SchemaId, schema_payload_location: PayloadLocation, block_number: BlockNumber,) ->
			Vec<MessageResponse> {
			Messages::get_messages_by_schema_and_block(schema_id, schema_payload_location, block_number)
		}

		fn get_schema_by_id(schema_id: SchemaId) -> Option<SchemaResponse> {
			Schemas::get_schema_by_id(schema_id)
		}
	}

	impl pallet_schemas_runtime_api::SchemasRuntimeApi<Block> for Runtime {
		fn get_by_schema_id(schema_id: SchemaId) -> Option<SchemaResponse> {
			Schemas::get_schema_by_id(schema_id)
		}
	}

	impl pallet_msa_runtime_api::MsaRuntimeApi<Block, AccountId> for Runtime {
		// *Temporarily Removed* until https://github.com/LibertyDSNP/frequency/issues/418 is completed
		// fn get_msa_keys(msa_id: MessageSourceId) -> Vec<KeyInfoResponse<AccountId>> {
		// 	Ok(Msa::fetch_msa_keys(msa_id))
		// }

		fn has_delegation(delegator: DelegatorId, provider: ProviderId, block_number: BlockNumber, schema_id: Option<SchemaId>) -> bool {
			match schema_id {
				Some(sid) => Msa::ensure_valid_schema_grant(provider, delegator, sid, block_number).is_ok(),
				None => Msa::ensure_valid_delegation(provider, delegator, Some(block_number)).is_ok(),
			}
		}

		fn get_granted_schemas_by_msa_id(delegator: DelegatorId, provider: ProviderId) -> Option<Vec<SchemaId>> {
			match Msa::get_granted_schemas_by_msa_id(delegator, provider) {
				Ok(x) => x,
				Err(_) => None,
			}
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade() -> (Weight, Weight) {
			log::info!("try-runtime::on_runtime_upgrade frequency.");
			let weight = Executive::try_runtime_upgrade().unwrap();
			(weight, RuntimeBlockWeights::get().max_block)
		}
		fn execute_block(block: Block, state_root_check: bool, select: frame_try_runtime::TryStateSelect) -> Weight {
			log::info!(
				target: "runtime::frequency", "try-runtime: executing block #{} ({:?}) / root checks: {:?} / sanity-checks: {:?}",
				block.header.number,
				block.header.hash(),
				state_root_check,
				select,
			);
			Executive::try_execute_block(block, state_root_check, select).expect("try_execute_block failed")
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;
			use frame_system_benchmarking::Pallet as SystemBench;
			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;
			use orml_benchmarking::list_benchmark as list_orml_benchmark;


			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);
			list_orml_benchmark!(list, extra, orml_vesting, benchmarking::vesting);

			let storage_info = AllPalletsWithSystem::storage_info();
			return (list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, TrackedStorageKey};

			use frame_system_benchmarking::Pallet as SystemBench;
			impl frame_system_benchmarking::Config for Runtime {}

			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;
			impl cumulus_pallet_session_benchmarking::Config for Runtime {}
			use orml_benchmarking::{add_benchmark as orml_add_benchmark};

			use frame_support::traits::WhitelistedStorageKeys;
			let whitelist: Vec<TrackedStorageKey> = AllPalletsWithSystem::whitelisted_storage_keys();

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);
			add_benchmarks!(params, batches);
			orml_add_benchmark!(params, batches, orml_vesting, benchmarking::vesting);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}
}

struct CheckInherents;

impl cumulus_pallet_parachain_system::CheckInherents<Block> for CheckInherents {
	fn check_inherents(
		block: &Block,
		relay_state_proof: &cumulus_pallet_parachain_system::RelayChainStateProof,
	) -> sp_inherents::CheckInherentsResult {
		let relay_chain_slot = relay_state_proof
			.read_slot()
			.expect("Could not read the relay chain slot from the proof");

		let inherent_data =
			cumulus_primitives_timestamp::InherentDataProvider::from_relay_chain_slot_and_duration(
				relay_chain_slot,
				sp_std::time::Duration::from_secs(6),
			)
			.create_inherent_data()
			.expect("Could not create the timestamp inherent data");

		inherent_data.check_extrinsics(block)
	}
}

cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
	CheckInherents = CheckInherents,
}

#[cfg(test)]
mod tests {
	use super::*;
	use frame_support::traits::WhitelistedStorageKeys;
	use sp_core::hexdisplay::HexDisplay;
	use std::collections::HashSet;

	#[test]
	fn check_whitelist() {
		let whitelist: HashSet<String> = dbg!(AllPalletsWithSystem::whitelisted_storage_keys()
			.iter()
			.map(|e| HexDisplay::from(&e.key).to_string())
			.collect());

		// Block Number
		assert!(
			whitelist.contains("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac")
		);
		// Total Issuance
		assert!(
			whitelist.contains("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80")
		);
		// Execution Phase
		assert!(
			whitelist.contains("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a")
		);
		// Event Count
		assert!(
			whitelist.contains("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850")
		);
		// System Events
		assert!(
			whitelist.contains("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7")
		);
	}
}
