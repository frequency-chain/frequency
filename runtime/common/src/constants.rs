use common_primitives::{
	node::{Balance, BlockNumber},
	schema::SchemaId,
};
use frame_support::{
	parameter_types,
	sp_runtime::{Perbill, Permill},
	traits::{ConstU32, ConstU8},
	weights::{constants::WEIGHT_PER_SECOND, Weight},
	PalletId,
};

pub const FREQUENCY_ROCOCO_TOKEN: &str = "XRQCY";
pub const FREQUENCY_TOKEN: &str = "FRQCY";

parameter_types! {
	/// Clone + Debug + Eq  implementation for u32 types
	pub const MaxDataSize: u32 = 30;
}

impl Clone for MaxDataSize {
	fn clone(&self) -> Self {
		MaxDataSize {}
	}
}

impl Eq for MaxDataSize {
	fn assert_receiver_is_total_eq(&self) {}
}

impl PartialEq for MaxDataSize {
	fn eq(&self, other: &Self) -> bool {
		self == other
	}
}

impl sp_std::fmt::Debug for MaxDataSize {
	#[cfg(feature = "std")]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}

	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

/// This determines the average expected block time that we are targeting.
/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
/// up by `pallet_aura` to implement `fn slot_duration()`.
///
/// Change this to adjust the block time.
pub const MILLISECS_PER_BLOCK: u64 = 12000;

// NOTE: Currently it is not possible to change the slot duration after the chain has started.
//       Attempting to do so will brick block production.
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

// Unit = the base number of indivisible units for balances
pub const UNIT: Balance = 100_000_000;
pub const MILLIUNIT: Balance = 1_000_000_000;
pub const MICROUNIT: Balance = 1_000_000;

/// The existential deposit. Set to 1/10 of the Connected Relay Chain.
pub const EXISTENTIAL_DEPOSIT: Balance = MILLIUNIT;

/// We assume that ~5% of the block weight is consumed by `on_initialize` handlers. This is
/// used to limit the maximal weight of a single extrinsic.
pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(5);

/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used by
/// `Operational` extrinsics.
pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

/// We allow for 0.5 of a second of compute with a 12 second average block time.
pub const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND.saturating_div(2);

pub type ZERO = ConstU32<0>;
pub type FIFTY = ConstU32<50>;

pub type FrameSystemMaxConsumers = ConstU32<16>;
pub type MsaMaxKeys = ConstU8<25>;
pub type MsaMaxProviderNameSize = ConstU32<16>;

parameter_types! {
	pub const SchemasMaxRegistrations: SchemaId = 65_000;
}
pub type SchemasMinModelSizeBytes = ConstU32<8>;
pub type SchemasMaxBytesBoundedVecLimit = ConstU32<65_500>;

parameter_types! {
	pub const VestingPalletId: PalletId = PalletId(*b"py/vstng");
}

parameter_types! {
	pub const MinVestedTransfer: Balance = 0;
}

pub const ORML_MAX_VESTING_SCHEDULES: u32 = 50;

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

pub type AuthorshipUncleGenerations = ZERO;

parameter_types! {
	pub const ExistentialDeposit: Balance = EXISTENTIAL_DEPOSIT;
}

pub type BalancesMaxLocks = FIFTY;
pub type BalancesMaxReserves = FIFTY;
pub type SchedulerMaxScheduledPerBlock = FIFTY;

pub type PreimageMaxSize = ConstU32<{ 4096 * 1024 }>;

parameter_types! {
	pub const PreimageBaseDeposit: Balance = 1 * MILLIUNIT;
	pub const PreimageByteDeposit: Balance = 1 * MICROUNIT;
}
pub type CouncilMaxProposals = ConstU32<25>;

parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = 5 * DAYS;
}

pub type TCMaxProposals = ConstU32<25>;
pub type TCMaxMembers = ConstU32<3>;

parameter_types! {
	pub const TCMotionDuration: BlockNumber = 5 * DAYS;
}

// Config from
// https://github.com/paritytech/substrate/blob/367dab0d4bd7fd7b6c222dd15c753169c057dd42/bin/node/runtime/src/lib.rs#L880
parameter_types! {
	pub const LaunchPeriod: BlockNumber = 28 * DAYS;
	pub const VotingPeriod: BlockNumber = 28 * DAYS;
	pub const FastTrackVotingPeriod: BlockNumber = 3 * DAYS;
	pub const EnactmentPeriod: BlockNumber = 30 * DAYS;
	pub const CooloffPeriod: BlockNumber = 28 * DAYS;
	pub const MinimumDeposit: Balance = 100 * UNIT;
}

pub type DemocracyMaxVotes = ConstU32<100>;
pub type DemocracyMaxProposals = FIFTY;

/// Generates the pallet "account"
/// 5EYCAe5ijiYfyeZ2JJCGq56LmPyNRAKzpG4QkoQkkQNB5e6Z
pub const TREASURY_PALLET_ID: PalletId = PalletId(*b"py/trsry");

// Treasury
// https://wiki.polkadot.network/docs/learn-treasury
// https://paritytech.github.io/substrate/master/pallet_treasury/pallet/trait.Config.html
parameter_types! {

	/// Keyless account that holds the money for the treasury
	pub const TreasuryPalletId: PalletId = TREASURY_PALLET_ID;

	/// Bond amount a treasury request must put up to make the proposal
	/// This will be transferred to OnSlash if the proposal is rejected
	pub const ProposalBondPercent: Permill = Permill::from_percent(5);

	/// Minimum bond for a treasury proposal
	pub const ProposalBondMinimum: Balance = 100 * UNIT;

	/// Minimum bond for a treasury proposal
	pub const ProposalBondMaximum: Balance = 1_000 * UNIT;

	/// How much of the treasury to burn, if funds remain at the end of the SpendPeriod
	/// Set to zero until the economic system is setup and stabilized
	pub const Burn: Permill = Permill::zero();

	/// Maximum number of approved proposals per Spending Period
	/// Set to 64 or 16 per week
	pub const MaxApprovals: u32 = 64;
}

pub type TransactionPaymentOperationalFeeMultiplier = ConstU8<5>;

parameter_types! {
	/// Relay Chain `TransactionByteFee` / 10
	pub const TransactionByteFee: Balance = 10 * MICROUNIT;
}

parameter_types! {
	pub const ReservedXcmpWeight: Weight =MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const ReservedDmpWeight: Weight =MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
}

pub type SessionPeriod = ConstU32<{ 6 * HOURS }>;
pub type SessionOffset = ZERO;

pub type AuraMaxAuthorities = ConstU32<100_000>;

pub type CollatorMaxCandidates = ZERO;
pub type CollatorMinCandidates = ZERO;

parameter_types! {
	pub const CollatorPotId: PalletId = PalletId(*b"PotStake");
	pub const MessagesMaxPayloadSizeBytes: u32 = 1024 * 50; // 50K
}

pub type MessagesMaxPerBlock = ConstU32<7000>;

impl Clone for MessagesMaxPayloadSizeBytes {
	fn clone(&self) -> Self {
		MessagesMaxPayloadSizeBytes {}
	}
}
