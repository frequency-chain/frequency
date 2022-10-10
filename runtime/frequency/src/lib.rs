#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub mod xcm_config;

mod benchmarking;

use cumulus_pallet_parachain_system::{
	RelayNumberStrictlyIncreases, RelaychainBlockNumberProvider,
};
use smallvec::smallvec;
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
pub use common_runtime::constants::*;

use frame_support::{
	construct_runtime,
	dispatch::DispatchError,
	parameter_types,
	traits::{ConstU32, Contains, EitherOfDiverse, EnsureOrigin, EqualPrivilegeOnly},
	weights::{
		constants::RocksDbWeight, ConstantMultiplier, DispatchClass, Weight,
		WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
	},
};
use frame_system::{
	limits::{BlockLength, BlockWeights},
	EnsureRoot, RawOrigin,
};
pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
pub use sp_runtime::{MultiAddress, Perbill, Permill};
use xcm_config::{XcmConfig, XcmOriginToTransactDispatchOrigin};

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

// XCM Imports
use xcm_executor::XcmExecutor;

/// Basefilter to only allow specified transactions call to be executed
pub struct BaseCallFilter;
impl Contains<Call> for BaseCallFilter {
	fn contains(call: &Call) -> bool {
		let core_calls = match call {
			Call::System(..) => true,
			Call::Timestamp(..) => true,
			Call::ParachainSystem(..) => true,
			Call::Sudo(..) => true,
			Call::TechnicalCommittee(..) => true,
			Call::Council(..) => true,
			Call::Democracy(..) => true,
			_ => false,
		};
		core_calls
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
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;

/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;

/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
>;

/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
/// node's balance type.
///
/// This should typically create a mapping between the following ranges:
///   - `[0, MAXIMUM_BLOCK_WEIGHT]`
///   - `[Balance::min, Balance::max]`
///
/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
///   - Setting it to `0` will essentially disable the weight fee.
///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
	type Balance = Balance;
	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		// in Rococo, extrinsic base weight (smallest non-zero weight) is mapped to 1 MILLIUNIT:
		// in our template, we map to 1/10 of that, or 1/10 MILLIUNIT
		let p = MILLIUNIT / 10;
		let q = 100 * Balance::from(ExtrinsicBaseWeight::get().ref_time());
		smallvec![WeightToFeeCoefficient {
			degree: 1,
			negative: false,
			coeff_frac: Perbill::from_rational(p % q, q),
			coeff_integer: p / q,
		}]
	}
}

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

#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("frequency"),
	impl_name: create_runtime_str!("frequency"),
	authoring_version: 1,
	spec_version: 1,
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
	pub const SS58Prefix: u16 = 42;
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// Base call filter to use in dispatchable.
	type BaseCallFilter = BaseCallFilter;
	/// The aggregated dispatch type that is available for extrinsics.
	type Call = Call;
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
	type Event = Event;
	/// The ubiquitous origin type.
	type Origin = Origin;
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
	type SS58Prefix = SS58Prefix;
	/// The action to take on a Runtime Upgrade
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	type MaxConsumers = FrameSystemMaxConsumers;
}

impl pallet_msa::Config for Runtime {
	type Event = Event;
	type WeightInfo = pallet_msa::weights::SubstrateWeight<Runtime>;
	type ConvertIntoAccountId32 = ConvertInto;
	type MaxKeys = MsaMaxKeys;
	type MaxSchemaGrants = MaxDataSize;
	type MaxProviderNameSize = MsaMaxProviderNameSize;
}

pub use common_primitives::schema::SchemaId;

impl pallet_schemas::Config for Runtime {
	type Event = Event;
	type WeightInfo = pallet_schemas::weights::SubstrateWeight<Runtime>;
	type MinSchemaModelSizeBytes = SchemasMinModelSizeBytes;
	type MaxSchemaRegistrations = SchemasMaxRegistrations;
	type SchemaModelMaxBytesBoundedVecLimit = SchemasMaxBytesBoundedVecLimit;
}

pub struct RootAsVestingPallet;
impl EnsureOrigin<Origin> for RootAsVestingPallet {
	type Success = AccountId;

	fn try_origin(o: Origin) -> Result<Self::Success, Origin> {
		Into::<Result<RawOrigin<AccountId>, Origin>>::into(o).and_then(|o| match o {
			RawOrigin::Root => Ok(VestingPalletId::get().into_account_truncating()),
			r => Err(Origin::from(r)),
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_origin() -> Origin {
		let zero_account_id =
			AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
				.expect("infinite length input; no invalid inputs for type; qed");
		Origin::from(RawOrigin::Signed(zero_account_id))
	}
}

parameter_types! {
	/// Need this declaration method for use + type safety in benchmarks
	pub const MaxVestingSchedules: u32 = ORML_MAX_VESTING_SCHEDULES;
}

impl orml_vesting::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type MinVestedTransfer = MinVestedTransfer;
	type VestedTransferOrigin = RootAsVestingPallet;
	type WeightInfo = ();
	type MaxVestingSchedules = MaxVestingSchedules;
	type BlockNumberProvider = RelaychainBlockNumberProvider<Runtime>;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type UncleGenerations = AuthorshipUncleGenerations;
	type FilterUncle = ();
	type EventHandler = (CollatorSelection,);
}

parameter_types! {
	pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = BalancesMaxLocks;
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
	type MaxReserves = BalancesMaxReserves;
	type ReserveIdentifier = [u8; 8];
}
parameter_types! {
	// The maximum weight that may be scheduled per block for any dispatchables of less priority than schedule::HARD_DEADLINE.
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(10) * RuntimeBlockWeights::get().max_block;
	// The maximum number of scheduled calls in the queue for a single block. Not strictly enforced, but used for weight estimation.
	// Retry a scheduled item every 25 blocks (5 minute) until the preimage exists.
	// Will be removed in v0.9.30
	pub const NoPreimagePostponement: Option<u32> = Some(5 * MINUTES);
}

// See also https://docs.rs/pallet-scheduler/latest/pallet_scheduler/trait.Config.html
impl pallet_scheduler::Config for Runtime {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = MaximumSchedulerWeight;
	/// Origin to schedule or cancel calls
	/// Set to Root or a simple majority of the Frequency Council
	type ScheduleOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 2>,
	>;
	type MaxScheduledPerBlock = SchedulerMaxScheduledPerBlock;
	type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Runtime>;
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type PreimageProvider = Preimage;
	// Will be removed in v0.9.30
	type NoPreimagePostponement = NoPreimagePostponement;
}

impl pallet_preimage::Config for Runtime {
	type WeightInfo = pallet_preimage::weights::SubstrateWeight<Runtime>;
	type Event = Event;
	type Currency = Balances;
	// Allow the Technical council to request preimages without deposit or fees
	type ManagerOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureMember<AccountId, TechnicalCommitteeInstance>,
	>;
	/// Expected to be removed in Polkadot v0.9.31
	type MaxSize = PreimageMaxSize;
	type BaseDeposit = PreimageBaseDeposit;
	type ByteDeposit = PreimageByteDeposit;
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = ConstU32<1>;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

type TechnicalCommitteeInstance = pallet_collective::Instance2;
impl pallet_collective::Config<TechnicalCommitteeInstance> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = TCMotionDuration;
	type MaxProposals = TCMaxProposals;
	type MaxMembers = TCMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	// TODO: this uses default but we don't have weights yet
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

// see https://paritytech.github.io/substrate/master/pallet_democracy/pallet/trait.Config.html
// for the definitions of these configs
impl pallet_democracy::Config for Runtime {
	type CooloffPeriod = CooloffPeriod;
	type Currency = Balances;
	type EnactmentPeriod = EnactmentPeriod;
	type Event = Event;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	type InstantAllowed = frame_support::traits::ConstBool<true>;
	type LaunchPeriod = LaunchPeriod;
	type MaxProposals = DemocracyMaxProposals;
	type MaxVotes = DemocracyMaxVotes;
	type MinimumDeposit = MinimumDeposit;
	type PreimageByteDeposit = PreimageByteDeposit;
	type Proposal = Call;
	type Scheduler = Scheduler;
	type Slash = (); // Treasury;
	type WeightInfo = pallet_democracy::weights::SubstrateWeight<Runtime>;
	type VoteLockingPeriod = EnactmentPeriod; // Same as EnactmentPeriod
	type VotingPeriod = VotingPeriod;

	// See https://paritytech.github.io/substrate/master/pallet_democracy/index.html for
	// the descriptions of these origins.
	/// A unanimous council can have the next scheduled referendum be a straight default-carries
	/// (NTB) vote.
	type ExternalDefaultOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 1>;

	/// A super-majority of 75% can have the next scheduled referendum be a straight majority-carries vote.
	type ExternalMajorityOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 4>;

	/// A straight majority (at least 50%) of the council can decide what their next motion is.
	type ExternalOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 2>;

	/// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
	/// be tabled immediately and with a shorter voting/enactment period.
	type FastTrackOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCommitteeInstance, 2, 3>;

	/// Origin from which the next majority-carries (or more permissive) referendum may be tabled to
	/// vote immediately and asynchronously in a similar manner to the emergency origin.
	/// Requires TechnicalCommittee to be unanimous.
	type InstantOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCommitteeInstance, 1, 1>;

	/// Overarching type of all pallets origins
	type PalletsOrigin = OriginCaller;

	/// To cancel a proposal which has been passed, 2/3 of the council must agree to it.
	type CancellationOrigin =
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 2, 3>;

	/// To cancel a proposal before it has been passed, the technical committee must be unanimous or
	/// Root must agree.
	type CancelProposalOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 1>,
	>;

	/// This origin can blacklist proposals.
	type BlacklistOrigin = EnsureRoot<AccountId>;

	/// Any single technical committee member may veto a coming council proposal, however they can
	/// only do it once and it lasts only for the cool-off period.
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCommitteeInstance>;

	type OperationalPreimageOrigin = pallet_collective::EnsureMember<AccountId, CouncilCollective>;
}

parameter_types! {
	/// The time proposals are collected before payout
	/// as well as when the burn of left over funds happens (if any)
	/// Setup on a 4-week scheduled for Mainnet
	pub const SpendPeriod: BlockNumber = 28 * DAYS;
}

impl pallet_treasury::Config for Runtime {
	/// Treasury Account: 5EYCAe5ijiYfyeZ2JJCGq56LmPyNRAKzpG4QkoQkkQNB5e6Z
	type PalletId = TreasuryPalletId;
	type Currency = Balances;
	type Event = Event;
	type WeightInfo = weights::pallet_treasury::WeightInfo<Runtime>;

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

impl pallet_transaction_payment::Config for Runtime {
	type Event = Event;
	type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
	type WeightToFee = WeightToFee;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
	type OperationalFeeMultiplier = TransactionPaymentOperationalFeeMultiplier;
}

parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
}

impl cumulus_pallet_parachain_system::Config for Runtime {
	type Event = Event;
	type OnSystemEvent = ();
	type SelfParaId = parachain_info::Pallet<Runtime>;
	type DmpMessageHandler = DmpQueue;
	type ReservedDmpWeight = ReservedDmpWeight;
	type OutboundXcmpMessageSource = XcmpQueue;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type CheckAssociatedRelayNumber = RelayNumberStrictlyIncreases;
}

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_aura_ext::Config for Runtime {}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = ();
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	type WeightInfo = ();
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}

// We allow root only to execute privileged collator selection operations.
pub type CollatorSelectionUpdateOrigin = EnsureRoot<AccountId>;

impl pallet_session::Config for Runtime {
	type Event = Event;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	// we don't have stash and controller, thus we don't need the convert as well.
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ShouldEndSession = pallet_session::PeriodicSessions<SessionPeriod, SessionOffset>;
	type NextSessionRotation = pallet_session::PeriodicSessions<SessionPeriod, SessionOffset>;
	type SessionManager = CollatorSelection;
	// Essentially just Aura, but lets be pedantic.
	type SessionHandler = <SessionKeys as sp_runtime::traits::OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	type WeightInfo = ();
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = AuraMaxAuthorities;
}

impl pallet_collator_selection::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type UpdateOrigin = CollatorSelectionUpdateOrigin;
	type PotId = CollatorPotId;
	type MaxCandidates = CollatorMaxCandidates;
	type MinCandidates = CollatorMinCandidates;
	type MaxInvulnerables = ConstU32<10>;
	// should be a multiple of session or things will get inconsistent
	type KickThreshold = SessionPeriod;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ValidatorRegistration = Session;
	type WeightInfo = ();
}

impl pallet_messages::Config for Runtime {
	type Event = Event;
	type WeightInfo = pallet_messages::weights::SubstrateWeight<Runtime>;
	type AccountProvider = Msa;
	type SchemaProvider = Schemas;
	type MaxMessagesPerBlock = MessagesMaxPerBlock;
	type MaxMessagePayloadSizeBytes = MessagesMaxPayloadSizeBytes;
}

impl pallet_sudo::Config for Runtime {
	type Event = Event;
	type Call = Call;
}

impl pallet_utility::Config for Runtime {
	type Event = Event;
	type Call = Call;
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

		// XCM helpers.
		XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>} = 30,
		PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin, Config} = 31,
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin} = 32,
		DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>} = 33,

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
		[cumulus_pallet_xcmp_queue, XcmpQueue]
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

	// Unfinished runtime APIs
	impl pallet_messages_runtime_api::MessagesApi<Block, BlockNumber> for Runtime {
		fn get_messages_by_schema_and_block(schema_id: SchemaId, schema_payload_location: PayloadLocation, block_number: BlockNumber,) ->
			Vec<MessageResponse<BlockNumber>> {
			Messages::get_messages_by_schema_and_block(schema_id, schema_payload_location, block_number)
		}

		fn get_schema_by_id(schema_id: SchemaId) -> Option<SchemaResponse> {
			Schemas::get_schema_by_id(schema_id)
		}
	}

	impl pallet_schemas_runtime_api::SchemasRuntimeApi<Block> for Runtime {
		fn get_by_schema_id(schema_id: SchemaId) -> Result<Option<SchemaResponse>, DispatchError> {
			Ok(Schemas::get_schema_by_id(schema_id))
		}
	}

	impl pallet_msa_runtime_api::MsaApi<Block, AccountId> for Runtime {
		// *Temporarily Removed* until https://github.com/LibertyDSNP/frequency/issues/418 is completed
		// fn get_msa_keys(msa_id: MessageSourceId) -> Result<Vec<KeyInfoResponse<AccountId>>, DispatchError> {
		// 	Ok(Msa::fetch_msa_keys(msa_id))
		// }

		fn has_delegation(delegator: Delegator, provider: Provider, block_number: Option<BlockNumber>) -> Result<bool, DispatchError> {
			match Msa::ensure_valid_delegation(provider, delegator, block_number) {
				Ok(_) => Ok(true),
				Err(_) => Err(sp_runtime::DispatchError::Other("Invalid Delegation")),
			}
		}

		fn get_granted_schemas(delegator: Delegator, provider: Provider) -> Result<Option<Vec<SchemaId>>, DispatchError> {
			Msa::get_granted_schemas(delegator, provider)
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

			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
				// Total Issuance
				hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
				// Execution Phase
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
				// Event Count
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
				// System Events
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
			];

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
