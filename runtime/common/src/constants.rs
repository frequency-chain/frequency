use crate::prod_or_testnet_or_local;
use common_primitives::node::BlockNumber;

use frame_support::{
	parameter_types,
	sp_runtime::{Perbill, Permill},
	traits::{ConstU128, ConstU16, ConstU32, ConstU64, ConstU8},
	weights::{constants::WEIGHT_REF_TIME_PER_SECOND, Weight},
	PalletId,
};

pub const FREQUENCY_ROCOCO_TOKEN: &str = "XRQCY";
pub const FREQUENCY_LOCAL_TOKEN: &str = "UNIT";
pub const FREQUENCY_TOKEN: &str = "FRQCY";
pub const TOKEN_DECIMALS: u8 = 8;

/// The maximum number of schema grants allowed per delegation
pub type MaxSchemaGrants = ConstU32<30>;

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
pub mod currency {
	use common_primitives::node::Balance;

	/// The existential deposit. Set to be 1/100th of a token.
	pub const EXISTENTIAL_DEPOSIT: Balance = CENTS;

	pub const UNITS: Balance = 10u128.saturating_pow(super::TOKEN_DECIMALS as u32);
	pub const DOLLARS: Balance = UNITS; // 100_000_000
	pub const CENTS: Balance = DOLLARS / 100; // 1_000_000
	pub const MILLICENTS: Balance = CENTS / 1_000; // 1_000

	/// Generates a balance based on amount of items and bytes
	/// Items are each worth 20 Dollars
	/// Bytes each cost 1/1_000 of a Dollar
	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 20 * DOLLARS + (bytes as Balance) * 100 * MILLICENTS
	}
}

/// We assume that ~5% of the block weight is consumed by `on_initialize` handlers. This is
/// used to limit the maximal weight of a single extrinsic.
pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(5);

/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used by
/// `Operational` extrinsics.
pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

/// We allow for 0.5 of a second of compute with a 12 second average block time.
pub const MAXIMUM_BLOCK_WEIGHT: Weight = Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND, 0)
	.saturating_div(2)
	.set_proof_size(cumulus_primitives_core::relay_chain::MAX_POV_SIZE as u64);
pub type ZERO = ConstU32<0>;
pub type FIFTY = ConstU32<50>;
pub type HUNDRED = ConstU32<100>;

// --- Frame System Pallet ---
pub type FrameSystemMaxConsumers = ConstU32<16>;
// -end- Frame System Pallet ---

// --- MSA Pallet ---
/// The maximum number of public keys per MSA
pub type MsaMaxPublicKeysPerMsa = ConstU8<25>;
/// The maximum size of the provider name (in bytes)
pub type MsaMaxProviderNameSize = ConstU32<16>;
/// The number of blocks per virtual bucket
pub type MSAMortalityWindowSize = ConstU32<100>;
/// The upper limit on total stored signatures.
/// Set to an average of 50 signatures per block
pub type MSAMaxSignaturesStored = ConstU32<50_000>;
// -end- MSA Pallet ---

// --- Schemas Pallet ---
/// The maximum number of schema registrations
pub type SchemasMaxRegistrations = ConstU16<65_000>;
/// The minimum schema model size (in bytes)
pub type SchemasMinModelSizeBytes = ConstU32<8>;
/// The maximum length of a schema model (in bytes)
pub type SchemasMaxBytesBoundedVecLimit = ConstU32<65_500>;
/// The maximum number of grants allowed per schema
pub type MaxSchemaSettingsPerSchema = ConstU32<2>;
// -end- Schemas Pallet ---

// --- Handles Pallet ---
// IMPORTANT: These values should only increase and never overlap with a previous set!
/// The minimum suffix value
pub type HandleSuffixMin = ConstU16<10>;
/// The maximum suffix value
pub type HandleSuffixMax = ConstU16<99>;
// -end- Handles Pallet

// --- TimeRelease Pallet ---
// Update
pub type MinReleaseTransfer = ConstU128<0>;

/// Update
pub const MAX_RELEASE_SCHEDULES: u32 = 50;
// -end- TimeRelease Pallet ---

// --- Timestamp Pallet ---
pub type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
// -end- Timestamp Pallet ---

// --- Authorship Pallet ---
pub type AuthorshipUncleGenerations = ZERO;
// -end- Authorship Pallet ---

// --- Balances Pallet ---
pub type BalancesMaxLocks = FIFTY;
pub type BalancesMaxReserves = FIFTY;
// -end- Balances Pallet ---

// --- Scheduler Pallet ---
pub type SchedulerMaxScheduledPerBlock = FIFTY;
// -end- Scheduler Pallet ---

// --- Preimage Pallet ---
/// Preimage maximum size set to 4 MB
/// Expected to be removed in Polkadot v0.9.31
pub type PreimageMaxSize = ConstU32<{ 4096 * 1024 }>;

pub type PreimageBaseDeposit = ConstU128<{ currency::deposit(10, 64) }>;
pub type PreimageByteDeposit = ConstU128<{ currency::deposit(0, 1) }>;
// -end- Preimage Pallet ---

// --- Council ---
// The maximum number of council proposals
pub type CouncilMaxProposals = ConstU32<25>;
// The maximum number of council members
pub type CouncilMaxMembers = ConstU32<10>;

pub type CouncilMotionDuration = ConstU32<{ 5 * DAYS }>;
// -end- Council ---

// --- Technical Committee ---
// The maximum number of technical committee proposals
pub type TCMaxProposals = ConstU32<25>;
// The maximum number of technical committee members
pub type TCMaxMembers = ConstU32<10>;

pub type TCMotionDuration = ConstU32<{ 5 * DAYS }>;
// -end- Technical Committee ---

// --- Democracy Pallet ---
// Config from
// https://github.com/paritytech/substrate/blob/367dab0d4bd7fd7b6c222dd15c753169c057dd42/bin/node/runtime/src/lib.rs#L880
pub type LaunchPeriod = ConstU32<{ prod_or_testnet_or_local!(7 * DAYS, 1 * DAYS, 5 * MINUTES) }>;
pub type VotingPeriod = ConstU32<{ prod_or_testnet_or_local!(7 * DAYS, 1 * DAYS, 5 * MINUTES) }>;
pub type FastTrackVotingPeriod =
	ConstU32<{ prod_or_testnet_or_local!(3 * HOURS, 30 * MINUTES, 5 * MINUTES) }>;
pub type EnactmentPeriod =
	ConstU32<{ prod_or_testnet_or_local!(8 * DAYS, 30 * HOURS, 10 * MINUTES) }>;
pub type CooloffPeriod = ConstU32<{ prod_or_testnet_or_local!(7 * DAYS, 1 * DAYS, 5 * MINUTES) }>;
pub type MinimumDeposit = ConstU128<
	{
		prod_or_testnet_or_local!(
			currency::deposit(5, 0),
			100 * currency::deposit(5, 0),
			100 * currency::deposit(5, 0)
		)
	},
>;
pub type SpendPeriod =
	ConstU32<{ prod_or_testnet_or_local!(7 * DAYS, 10 * MINUTES, 10 * MINUTES) }>;
pub type DemocracyMaxVotes = ConstU32<100>;
pub type DemocracyMaxProposals = HUNDRED;
// -end- Democracy Pallet ---

// --- Treasury Pallet ---
/// Generates the pallet "account"
/// 5EYCAe5ijiYfyeZ2JJCGq56LmPyNRAKzpG4QkoQkkQNB5e6Z
pub const TREASURY_PALLET_ID: PalletId = PalletId(*b"py/trsry");

// https://wiki.polkadot.network/docs/learn-treasury
// https://paritytech.github.io/substrate/master/pallet_treasury/pallet/trait.Config.html
// Needs parameter_types! for the Permill and PalletId
parameter_types! {

	/// Keyless account that holds the money for the treasury
	pub const TreasuryPalletId: PalletId = TREASURY_PALLET_ID;

	/// Bond amount a treasury request must put up to make the proposal
	/// This will be transferred to OnSlash if the proposal is rejected
	pub const ProposalBondPercent: Permill = Permill::from_percent(5);

	/// How much of the treasury to burn, if funds remain at the end of the SpendPeriod
	/// Set to zero until the economic system is setup and stabilized
	pub const Burn: Permill = Permill::zero();
}

/// Maximum number of approved proposals per Spending Period
/// Set to 64 or 16 per week
pub type MaxApprovals = ConstU32<64>;

/// Minimum bond for a treasury proposal
pub type ProposalBondMinimum = ConstU128<{ 100 * currency::DOLLARS }>;

/// Minimum bond for a treasury proposal
pub type ProposalBondMaximum = ConstU128<{ 1_000 * currency::DOLLARS }>;

// -end- Treasury Pallet ---

// --- Transaction Payment Pallet ---
// The fee multiplier
pub type TransactionPaymentOperationalFeeMultiplier = ConstU8<5>;

/// Relay Chain `TransactionByteFee` / 10
pub type TransactionByteFee = ConstU128<{ 10 * currency::MILLICENTS }>;
// -end- Transaction Payment Pallet ---

// --- Frequency Transaction Payment Pallet ---
pub type MaximumCapacityBatchLength = ConstU8<10>;
// -end- Frequency Transaction Payment Pallet ---

// --- Session Pallet ---
pub type SessionPeriod = ConstU32<{ 6 * HOURS }>;
pub type SessionOffset = ZERO;
// -end- Session Pallet ---

// --- Aura Pallet ---
/// The maximum number of authorities
pub type AuraMaxAuthorities = ConstU32<100_000>;
// -end- Aura Pallet ---

// --- Collator Selection Pallet ---
// Values for each runtime environment are independently configurable.
// Example CollatorMaxInvulnerables are 16 in production(mainnet),
// 5 in rococo testnet and 5 in rococo local

pub type CollatorMaxCandidates = ConstU32<50>;
pub type CollatorMinCandidates = ConstU32<1>;
pub type CollatorMaxInvulnerables = ConstU32<{ prod_or_testnet_or_local!(16, 5, 5) }>;
pub type CollatorKickThreshold =
	ConstU32<{ prod_or_testnet_or_local!(6 * HOURS, 6 * HOURS, 6 * HOURS) }>;

// Needs parameter_types! for the PalletId and impls below
parameter_types! {
	pub const NeverDepositIntoId: PalletId = PalletId(*b"NeverDep");
	pub const MessagesMaxPayloadSizeBytes: u32 = 1024 * 50; // 50K
}
// -end- Collator Selection Pallet ---

// --- Messages Pallet ---
/// The maximum number of messages per block
pub type MessagesMaxPerBlock = ConstU32<7000>;

impl Clone for MessagesMaxPayloadSizeBytes {
	fn clone(&self) -> Self {
		MessagesMaxPayloadSizeBytes {}
	}
}
// -end- Messages Pallet ---

// Needs parameter_types! to reduce pallet dependencies
parameter_types! {
	/// SS58 Prefix for the for Frequency Network
	/// 90 is the prefix for the Frequency Network on Polkadot
	/// 42 is the prefix for the Frequency Network on Rococo
	pub const Ss58Prefix: u16 = prod_or_testnet_or_local!(90, 42, 42);
}

// --- Stateful Storage Pallet ---
// Needs parameter_types! for the impls below
parameter_types! {
	/// The maximum size of a page (in bytes) for an Itemized storage model (64KB)
	pub const MaxItemizedPageSizeBytes: u32 = 64 * 1024;
	/// The maximum size of a page (in bytes) for a Paginated storage model (1KB)
	pub const MaxPaginatedPageSizeBytes: u32 = 1 * 1024;
	/// The maximum size of a single item in an itemized storage model (in bytes)
	pub const MaxItemizedBlobSizeBytes: u32 = 1024;
}
/// The maximum number of pages in a Paginated storage model
pub type MaxPaginatedPageId = ConstU16<16>;
/// The maximum number of actions in itemized actions
pub type MaxItemizedActionsCount = ConstU32<5>;
/// The number of blocks for Stateful mortality is 24 hours
pub type StatefulMortalityWindowSize = ConstU32<14400>;
// -end- Stateful Storage Pallet

impl Default for MaxItemizedPageSizeBytes {
	fn default() -> Self {
		Self
	}
}

impl Default for MaxPaginatedPageSizeBytes {
	fn default() -> Self {
		Self
	}
}

impl Clone for MaxItemizedBlobSizeBytes {
	fn clone(&self) -> Self {
		MaxItemizedBlobSizeBytes {}
	}
}

impl Eq for MaxItemizedBlobSizeBytes {
	fn assert_receiver_is_total_eq(&self) {}
}

impl PartialEq for MaxItemizedBlobSizeBytes {
	fn eq(&self, other: &Self) -> bool {
		self == other
	}
}

impl sp_std::fmt::Debug for MaxItemizedBlobSizeBytes {
	#[cfg(feature = "std")]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}

	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

// --- Capacity Pallet ---
pub type CapacityMinimumStakingAmount = ConstU128<{ currency::EXISTENTIAL_DEPOSIT }>;
pub type CapacityMinimumTokenBalance = ConstU128<{ currency::DOLLARS }>;
pub type CapacityMaxUnlockingChunks = ConstU32<4>;
pub type CapacityMaxEpochLength = ConstU32<7_200>; // one day, assuming 12 second blocks.

#[cfg(not(any(feature = "frequency-rococo-local", feature = "frequency-no-relay")))]
pub type CapacityUnstakingThawPeriod = ConstU16<30>; // 30 Epochs, or 30 days given the above

#[cfg(any(feature = "frequency-rococo-local", feature = "frequency-no-relay"))]
pub type CapacityUnstakingThawPeriod = ConstU16<2>; // 2 Epochs

// Needs parameter_types! for the Perbil
parameter_types! {
	// 1:50 Capacity:Token, must be declared this way instead of using `from_rational` because of
	//  ```error[E0015]: cannot call non-const fn `Perbill::from_rational::<u32>` in constant functions```
	pub const CapacityPerToken: Perbill = Perbill::from_percent(2);
}

pub type CapacityChangeStakingTargetThawEras = ConstU32<5>;

// -end- Capacity Pallet ---
