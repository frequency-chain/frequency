#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

extern crate alloc;
#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking; // Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

#[cfg(feature = "std")]
#[allow(clippy::expect_used)]
/// Wasm binary unwrapped. If built with `WASM_BINARY`, the function panics.
pub fn wasm_binary_unwrap() -> &'static [u8] {
	WASM_BINARY.expect(
		"wasm binary is not available. This means the client is \
                        built with `WASM_BINARY` flag and it is only usable for \
                        production chains. Please rebuild with the flag disabled.",
	)
}

#[cfg(feature = "frequency-bridging")]
pub mod xcm;

#[cfg(feature = "frequency-bridging")]
use frame_support::traits::AsEnsureOriginWithArg;

#[cfg(feature = "frequency-bridging")]
use frame_system::EnsureNever;

#[cfg(feature = "frequency-bridging")]
use xcm::{
	parameters::{
		ForeignAssetsAssetId, NativeToken, RelayLocation, RelayOrigin, ReservedDmpWeight,
		ReservedXcmpWeight,
	},
	queue::XcmRouter,
	LocationToAccountId, XcmConfig,
};

#[cfg(test)]
mod migration_tests;

use alloc::borrow::Cow;
use common_runtime::constants::currency::UNITS;

#[cfg(feature = "frequency-bridging")]
use staging_xcm::{
	prelude::AssetId as AssetLocationId, Version as XcmVersion, VersionedAsset, VersionedAssetId,
	VersionedAssets, VersionedLocation, VersionedXcm,
};

#[cfg(feature = "frequency-bridging")]
use xcm_runtime_apis::{
	dry_run::{CallDryRunEffects, Error as XcmDryRunApiError, XcmDryRunEffects},
	fees::Error as XcmPaymentApiError,
};

#[cfg(any(
	not(feature = "frequency-no-relay"),
	feature = "frequency-lint-check",
	feature = "frequency-bridging"
))]
use cumulus_pallet_parachain_system::{
	DefaultCoreSelector, RelayNumberMonotonicallyIncreases, RelaychainDataProvider,
};
#[cfg(any(feature = "runtime-benchmarks", feature = "test"))]
use frame_support::traits::MapSuccess;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
#[cfg(any(feature = "runtime-benchmarks", feature = "test"))]
use sp_runtime::traits::Replace;
use sp_runtime::{
	generic, impl_opaque_keys,
	traits::{AccountIdConversion, BlakeTwo256, Block as BlockT, ConvertInto, IdentityLookup},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, DispatchError,
};

use pallet_collective::Members;

#[cfg(any(feature = "runtime-benchmarks", feature = "test"))]
use pallet_collective::ProposalCount;

use parity_scale_codec::Encode;

#[cfg(feature = "std")]
use sp_version::NativeVersion;

use sp_version::RuntimeVersion;
use static_assertions::const_assert;

use common_primitives::{
	handles::{
		BaseHandle, CheckHandleResponse, DisplayHandle, HandleResponse, PresumptiveSuffixesResponse,
	},
	messages::MessageResponse,
	msa::{
		AccountId20Response, DelegationResponse, DelegationValidator, DelegatorId, MessageSourceId,
		ProviderId, SchemaGrant, SchemaGrantValidator, H160,
	},
	node::{
		AccountId, Address, Balance, BlockNumber, Hash, Header, Index, ProposalProvider, Signature,
		UtilityProvider,
	},
	rpc::RpcEvent,
	schema::{PayloadLocation, SchemaId, SchemaResponse, SchemaVersionResponse},
	stateful_storage::{ItemizedStoragePageResponse, PaginatedStorageResponse},
};

pub use common_runtime::{
	constants::{
		currency::{CENTS, EXISTENTIAL_DEPOSIT},
		*,
	},
	fee::WeightToFee,
	prod_or_testnet_or_local,
	proxy::ProxyType,
};

use frame_support::{
	construct_runtime,
	dispatch::{DispatchClass, GetDispatchInfo, Pays},
	genesis_builder_helper::{build_state, get_preset},
	pallet_prelude::DispatchResultWithPostInfo,
	parameter_types,
	traits::{
		fungible::HoldConsideration,
		schedule::LOWEST_PRIORITY,
		tokens::{PayFromAccount, UnityAssetBalanceConversion},
		ConstBool, ConstU128, ConstU32, ConstU64, EitherOfDiverse, EnsureOrigin,
		EqualPrivilegeOnly, GetStorageVersion, InstanceFilter, LinearStoragePrice,
		OnRuntimeUpgrade,
	},
	weights::{constants::WEIGHT_REF_TIME_PER_SECOND, ConstantMultiplier, Weight},
	Twox128,
};

use frame_system::{
	limits::{BlockLength, BlockWeights},
	EnsureRoot, EnsureSigned,
};

use alloc::{boxed::Box, vec, vec::Vec};

pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
pub use sp_runtime::Perbill;

#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;

pub use pallet_capacity;
pub use pallet_frequency_tx_payment::{capacity_stable_weights, types::GetStableWeight};
pub use pallet_msa;
pub use pallet_passkey;
pub use pallet_schemas;
pub use pallet_time_release::types::{ScheduleName, SchedulerProviderTrait};

// Polkadot Imports
use polkadot_runtime_common::{BlockHashCount, SlowAdjustingFeeUpdate};

use common_primitives::{capacity::UnclaimedRewardInfo, schema::NameLookupResponse};
use common_runtime::weights::rocksdb_weights::constants::RocksDbWeight;
pub use common_runtime::{
	constants::MaxSchemaGrants,
	weights,
	weights::{block_weights::BlockExecutionWeight, extrinsic_weights::ExtrinsicBaseWeight},
};
use frame_support::traits::Contains;
#[cfg(feature = "try-runtime")]
use frame_support::traits::{TryStateSelect, UpgradeCheckSelect};

mod ethereum;
mod genesis;

pub mod polkadot_xcm_fee {
	use crate::{Balance, ExtrinsicBaseWeight, WEIGHT_REF_TIME_PER_SECOND};
	pub const MICRO_DOT: Balance = 10_000;
	pub const MILLI_DOT: Balance = 1_000 * MICRO_DOT;

	pub fn default_fee_per_second() -> u128 {
		let base_weight = Balance::from(ExtrinsicBaseWeight::get().ref_time());
		let base_tx_per_second = (WEIGHT_REF_TIME_PER_SECOND as u128) / base_weight;
		base_tx_per_second * base_relay_tx_fee()
	}

	pub fn base_relay_tx_fee() -> Balance {
		MILLI_DOT
	}
}

pub struct SchedulerProvider;

impl SchedulerProviderTrait<RuntimeOrigin, BlockNumber, RuntimeCall> for SchedulerProvider {
	fn schedule(
		origin: RuntimeOrigin,
		id: ScheduleName,
		when: BlockNumber,
		call: Box<RuntimeCall>,
	) -> Result<(), DispatchError> {
		Scheduler::schedule_named(origin, id, when, None, LOWEST_PRIORITY, call)?;

		Ok(())
	}

	fn cancel(origin: RuntimeOrigin, id: [u8; 32]) -> Result<(), DispatchError> {
		Scheduler::cancel_named(origin, id)?;

		Ok(())
	}
}

pub struct CouncilProposalProvider;

impl ProposalProvider<AccountId, RuntimeCall> for CouncilProposalProvider {
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
		let members = Members::<Runtime, CouncilCollective>::get();
		let threshold: u32 = ((members.len() / 2) + 1) as u32;
		let length_bound: u32 = proposal.using_encoded(|p| p.len() as u32);
		Council::do_propose_proposed(who, threshold, proposal, length_bound)
	}

	#[cfg(any(feature = "runtime-benchmarks", feature = "test"))]
	fn proposal_count() -> u32 {
		ProposalCount::<Runtime, CouncilCollective>::get()
	}
}

pub struct CapacityBatchProvider;

impl UtilityProvider<RuntimeOrigin, RuntimeCall> for CapacityBatchProvider {
	fn batch_all(origin: RuntimeOrigin, calls: Vec<RuntimeCall>) -> DispatchResultWithPostInfo {
		Utility::batch_all(origin, calls)
	}
}

/// Base filter to only allow calls to specified transactions to be executed
pub struct BaseCallFilter;

impl Contains<RuntimeCall> for BaseCallFilter {
	fn contains(call: &RuntimeCall) -> bool {
		match call {
			RuntimeCall::Utility(pallet_utility_call) =>
				Self::is_utility_call_allowed(pallet_utility_call),

			#[cfg(feature = "frequency")]
			// Filter out calls that are Governance actions on Mainnet
			RuntimeCall::Msa(pallet_msa::Call::create_provider { .. }) |
			RuntimeCall::Schemas(pallet_schemas::Call::create_schema_v3 { .. }) |
			RuntimeCall::Schemas(pallet_schemas::Call::create_intent { .. }) |
			RuntimeCall::Schemas(pallet_schemas::Call::create_intent_group { .. }) |
			RuntimeCall::Schemas(pallet_schemas::Call::update_intent_group { .. }) => false,

			#[cfg(all(feature = "frequency-bridging", feature = "frequency"))]
			RuntimeCall::PolkadotXcm(pallet_xcm_call) => Self::is_xcm_call_allowed(pallet_xcm_call),
			// Everything else is allowed
			_ => true,
		}
	}
}

impl BaseCallFilter {
	#[cfg(all(feature = "frequency", feature = "frequency-bridging"))]
	fn is_xcm_call_allowed(call: &pallet_xcm::Call<Runtime>) -> bool {
		!matches!(
			call,
			pallet_xcm::Call::transfer_assets { .. } |
				pallet_xcm::Call::teleport_assets { .. } |
				pallet_xcm::Call::limited_teleport_assets { .. } |
				pallet_xcm::Call::reserve_transfer_assets { .. } |
				pallet_xcm::Call::add_authorized_alias { .. } |
				pallet_xcm::Call::remove_authorized_alias { .. } |
				pallet_xcm::Call::remove_all_authorized_aliases { .. }
		)
	}

	fn is_utility_call_allowed(call: &pallet_utility::Call<Runtime>) -> bool {
		match call {
			pallet_utility::Call::batch { calls, .. } |
			pallet_utility::Call::batch_all { calls, .. } |
			pallet_utility::Call::force_batch { calls, .. } => calls.iter().any(Self::is_batch_call_allowed),
			_ => true,
		}
	}

	fn is_batch_call_allowed(call: &RuntimeCall) -> bool {
		match call {
			// Block all nested `batch` calls from utility batch
			RuntimeCall::Utility(pallet_utility::Call::batch { .. }) |
			RuntimeCall::Utility(pallet_utility::Call::batch_all { .. }) |
			RuntimeCall::Utility(pallet_utility::Call::force_batch { .. }) => false,

			// Block all `FrequencyTxPayment` calls from utility batch
			RuntimeCall::FrequencyTxPayment(..) => false,

			#[cfg(feature = "frequency")]
			// Block calls from utility (or Capacity) batch that are Governance actions on Mainnet
			RuntimeCall::Msa(pallet_msa::Call::create_provider { .. }) |
			RuntimeCall::Schemas(pallet_schemas::Call::create_schema_v3 { .. }) |
			RuntimeCall::Schemas(pallet_schemas::Call::create_intent { .. }) |
			RuntimeCall::Schemas(pallet_schemas::Call::create_intent_group { .. }) |
			RuntimeCall::Schemas(pallet_schemas::Call::update_intent_group { .. }) => false,

			// Block `Pays::No` calls from utility batch
			_ if Self::is_pays_no_call(call) => false,

			// Allow all other calls
			_ => true,
		}
	}

	fn is_pays_no_call(call: &RuntimeCall) -> bool {
		call.get_dispatch_info().pays_fee == Pays::No
	}
}

// Proxy Pallet Filters
impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::NonTransfer => matches!(
				c,
				// Sorted
				// Skip: RuntimeCall::Balances
				RuntimeCall::Capacity(..)
				| RuntimeCall::CollatorSelection(..)
				| RuntimeCall::Council(..)
				| RuntimeCall::Democracy(..)
				| RuntimeCall::FrequencyTxPayment(..) // Capacity Tx never transfer
				| RuntimeCall::Handles(..)
				| RuntimeCall::Messages(..)
				| RuntimeCall::Msa(..)
				| RuntimeCall::Multisig(..)
				// Skip: ParachainSystem(..)
				| RuntimeCall::Preimage(..)
				| RuntimeCall::Scheduler(..)
				| RuntimeCall::Schemas(..)
				| RuntimeCall::Session(..)
				| RuntimeCall::StatefulStorage(..)
				// Skip: RuntimeCall::Sudo
				// Skip: RuntimeCall::System
				| RuntimeCall::TechnicalCommittee(..)
				// Specifically omitting TimeRelease `transfer`, and `update_release_schedules`
				| RuntimeCall::TimeRelease(pallet_time_release::Call::claim{..})
				| RuntimeCall::TimeRelease(pallet_time_release::Call::claim_for{..})
				// Skip: RuntimeCall::Timestamp
				| RuntimeCall::Treasury(..)
				| RuntimeCall::Utility(..) // Calls inside a batch are also run through filters
			),
			ProxyType::Governance => matches!(
				c,
				RuntimeCall::Treasury(..) |
					RuntimeCall::Democracy(..) |
					RuntimeCall::TechnicalCommittee(..) |
					RuntimeCall::Council(..) |
					RuntimeCall::Utility(..) // Calls inside a batch are also run through filters
			),
			ProxyType::Staking => {
				matches!(
					c,
					RuntimeCall::Capacity(pallet_capacity::Call::stake { .. }) |
						RuntimeCall::CollatorSelection(
							pallet_collator_selection::Call::set_candidacy_bond { .. }
						)
				)
			},
			ProxyType::CancelProxy => {
				matches!(c, RuntimeCall::Proxy(pallet_proxy::Call::reject_announcement { .. }))
			},
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(ProxyType::Any, _) => true,
			(_, ProxyType::Any) => false,
			(ProxyType::NonTransfer, _) => true,
			_ => false,
		}
	}
}

/// PasskeyCallFilter to only allow calls to specified transactions to be executed
pub struct PasskeyCallFilter;

impl Contains<RuntimeCall> for PasskeyCallFilter {
	fn contains(call: &RuntimeCall) -> bool {
		match call {
			#[cfg(feature = "runtime-benchmarks")]
			RuntimeCall::System(frame_system::Call::remark { .. }) => true,

			RuntimeCall::Balances(_) | RuntimeCall::Capacity(_) => true,
			_ => false,
		}
	}
}

pub struct MsaCallFilter;
use pallet_frequency_tx_payment::types::GetAddKeyData;
impl GetAddKeyData<RuntimeCall, AccountId, MessageSourceId> for MsaCallFilter {
	fn get_add_key_data(call: &RuntimeCall) -> Option<(AccountId, AccountId, MessageSourceId)> {
		match call {
			RuntimeCall::Msa(MsaCall::add_public_key_to_msa {
				add_key_payload,
				new_key_owner_proof: _,
				msa_owner_public_key,
				msa_owner_proof: _,
			}) => {
				let new_key = add_key_payload.clone().new_public_key;
				Some((msa_owner_public_key.clone(), new_key, add_key_payload.msa_id))
			},
			_ => None,
		}
	}
}

/// The TransactionExtension to the basic transaction logic.
pub type TxExtension = cumulus_pallet_weight_reclaim::StorageWeightReclaim<
	Runtime,
	(
		frame_system::CheckNonZeroSender<Runtime>,
		// merging these types so that we can have more than 12 extensions
		(frame_system::CheckSpecVersion<Runtime>, frame_system::CheckTxVersion<Runtime>),
		frame_system::CheckGenesis<Runtime>,
		frame_system::CheckEra<Runtime>,
		common_runtime::extensions::check_nonce::CheckNonce<Runtime>,
		pallet_frequency_tx_payment::ChargeFrqTransactionPayment<Runtime>,
		pallet_msa::CheckFreeExtrinsicUse<Runtime>,
		pallet_handles::handles_signed_extension::HandlesSignedExtension<Runtime>,
		frame_metadata_hash_extension::CheckMetadataHash<Runtime>,
		frame_system::CheckWeight<Runtime>,
	),
>;

/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;

/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

#[cfg(feature = "frequency-bridging")]
pub type AssetBalance = Balance;

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, TxExtension>;

/// Executive: handles dispatch to the various modules.
#[cfg(feature = "frequency-bridging")]
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
	(MigratePalletsCurrentStorage<Runtime>, SetSafeXcmVersion<Runtime>),
>;

#[cfg(not(feature = "frequency-bridging"))]
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
	(MigratePalletsCurrentStorage<Runtime>,),
>;

pub struct MigratePalletsCurrentStorage<T>(core::marker::PhantomData<T>);

impl<T: pallet_collator_selection::Config> OnRuntimeUpgrade for MigratePalletsCurrentStorage<T> {
	fn on_runtime_upgrade() -> Weight {
		use sp_core::Get;

		if pallet_collator_selection::Pallet::<T>::on_chain_storage_version() !=
			pallet_collator_selection::Pallet::<T>::in_code_storage_version()
		{
			pallet_collator_selection::Pallet::<T>::in_code_storage_version()
				.put::<pallet_collator_selection::Pallet<T>>();

			log::info!("Setting version on pallet_collator_selection");
		}

		T::DbWeight::get().reads_writes(1, 1)
	}
}

/// Migration to set the initial safe XCM version for the XCM pallet.
pub struct SetSafeXcmVersion<T>(core::marker::PhantomData<T>);

#[cfg(feature = "frequency-bridging")]
use common_runtime::constants::xcm_version::SAFE_XCM_VERSION;

#[cfg(feature = "frequency-bridging")]
impl<T: pallet_xcm::Config> OnRuntimeUpgrade for SetSafeXcmVersion<T> {
	fn on_runtime_upgrade() -> Weight {
		use sp_core::Get;

		// Access storage directly using storage key because `pallet_xcm` does not provide a direct API to get the safe XCM version.
		let storage_key = frame_support::storage::storage_prefix(b"PolkadotXcm", b"SafeXcmVersion");
		log::info!("Checking SafeXcmVersion in storage with key: {storage_key:?}");

		let current_version = frame_support::storage::unhashed::get::<u32>(&storage_key);
		match current_version {
			Some(version) if version == SAFE_XCM_VERSION => {
				log::info!("SafeXcmVersion already set to {version}, skipping migration.");
				T::DbWeight::get().reads(1)
			},
			Some(version) => {
				log::info!(
					"SafeXcmVersion currently set to {version}, updating to {SAFE_XCM_VERSION}"
				);
				// Set the safe XCM version directly in storage
				frame_support::storage::unhashed::put(&storage_key, &(SAFE_XCM_VERSION));
				T::DbWeight::get().reads(1).saturating_add(T::DbWeight::get().writes(1))
			},
			None => {
				log::info!("SafeXcmVersion not set, setting to {SAFE_XCM_VERSION}");
				// Set the safe XCM version directly in storage
				frame_support::storage::unhashed::put(&storage_key, &(SAFE_XCM_VERSION));
				T::DbWeight::get().reads(1).saturating_add(T::DbWeight::get().writes(1))
			},
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
		use parity_scale_codec::Encode;

		// Check pallet state before migration
		pallet_xcm::Pallet::<T>::do_try_state()?;
		log::info!("pre_upgrade: PolkadotXcm pallet state is valid before migration");

		// Read the actual current SafeXcmVersion from storage
		let storage_key = frame_support::storage::storage_prefix(b"PolkadotXcm", b"SafeXcmVersion");
		let current_version = frame_support::storage::unhashed::get::<u32>(&storage_key);

		log::info!("pre_upgrade: Current SafeXcmVersion = {:?}", current_version);

		// Return the actual current state encoded for post_upgrade verification
		Ok(current_version.encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
		use parity_scale_codec::Decode;

		// Decode the pre-upgrade state
		let pre_upgrade_version = Option::<u32>::decode(&mut &state[..])
			.map_err(|_| "Failed to decode pre-upgrade state")?;

		let storage_key = frame_support::storage::storage_prefix(b"PolkadotXcm", b"SafeXcmVersion");
		let current_version = frame_support::storage::unhashed::get::<u32>(&storage_key);

		log::info!(
			"post_upgrade: Pre-upgrade version = {:?}, Current version = {:?}",
			pre_upgrade_version,
			current_version
		);

		// Verify the migration worked correctly
		match current_version {
			Some(version) if version == SAFE_XCM_VERSION => {
				log::info!(
					"post_upgrade: Migration successful - SafeXcmVersion correctly set to {}",
					version
				);
			},
			Some(version) => {
				log::error!("post_upgrade: Migration failed - SafeXcmVersion was set to {}, but expected {}", version, SAFE_XCM_VERSION);
				return Err(sp_runtime::TryRuntimeError::Other(
					"SafeXcmVersion was set to incorrect version after migration",
				));
			},
			None => {
				return Err(sp_runtime::TryRuntimeError::Other(
					"SafeXcmVersion should be set after migration but found None",
				));
			},
		}

		// Check pallet state after migration
		pallet_xcm::Pallet::<T>::do_try_state()?;
		log::info!("post_upgrade: PolkadotXcm pallet state is valid after migration");

		Ok(())
	}
}

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	use super::*;
	use sp_runtime::{
		generic,
		traits::{BlakeTwo256, Hash as HashT},
	};

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;
	/// Opaque block hash type.
	pub type Hash = <BlakeTwo256 as HashT>::Output;
}

impl_opaque_keys! {
	pub struct SessionKeys {
		pub aura: Aura,
	}
}

// IMPORTANT: Remember to update spec_version in BOTH structs below
#[cfg(feature = "frequency")]
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: Cow::Borrowed("frequency"),
	impl_name: Cow::Borrowed("frequency"),
	authoring_version: 1,
	spec_version: 177,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	system_version: 1,
};

// IMPORTANT: Remember to update spec_version in above struct too
#[cfg(not(feature = "frequency"))]
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: Cow::Borrowed("frequency-testnet"),
	impl_name: Cow::Borrowed("frequency"),
	authoring_version: 1,
	spec_version: 177,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	system_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

// Needs parameter_types! for the complex logic
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

// ---------- Foreign Assets pallet parameters ----------
#[cfg(feature = "frequency-bridging")]
parameter_types! {
	pub const AssetDeposit: Balance = 0;
	pub const AssetAccountDeposit: Balance = 0;
	pub const MetadataDepositBase: Balance = 0;
	pub const MetadataDepositPerByte: Balance = 0;
	pub const ApprovalDeposit: Balance = 0;
	pub const AssetsStringLimit: u32 = 50;

	// we just reuse the same deposits
	pub const ForeignAssetsAssetDeposit: Balance = AssetDeposit::get();
	pub const ForeignAssetsAssetAccountDeposit: Balance = AssetAccountDeposit::get();
	pub const ForeignAssetsApprovalDeposit: Balance = ApprovalDeposit::get();
	pub const ForeignAssetsAssetsStringLimit: u32 = AssetsStringLimit::get();
	pub const ForeignAssetsMetadataDepositBase: Balance = MetadataDepositBase::get();
	pub const ForeignAssetsMetadataDepositPerByte: Balance = MetadataDepositPerByte::get();
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
	type RuntimeTask = RuntimeTask;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// Base call filter to use in dispatchable.
	// enable for cfg feature "frequency" only
	type BaseCallFilter = BaseCallFilter;
	/// The aggregated dispatch type that is available for extrinsics.
	type RuntimeCall = RuntimeCall;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = EthereumCompatibleAccountIdLookup<AccountId, ()>;
	/// The index type for storing how many extrinsics an account has signed.
	type Nonce = Index;
	/// The block type.
	type Block = Block;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
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
	#[cfg(any(
		not(feature = "frequency-no-relay"),
		feature = "frequency-lint-check",
		feature = "frequency-bridging"
	))]
	type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
	#[cfg(feature = "frequency-no-relay")]
	type OnSetCode = ();
	type MaxConsumers = FrameSystemMaxConsumers;
	///  A new way of configuring migrations that run in a single block.
	type SingleBlockMigrations = ();
	/// The migrator that is used to run Multi-Block-Migrations.
	type MultiBlockMigrator = ();
	/// A callback that executes in *every block* directly before all inherents were applied.
	type PreInherents = ();
	/// A callback that executes in *every block* directly after all inherents were applied.
	type PostInherents = ();
	/// A callback that executes in *every block* directly after all transactions were applied.
	type PostTransactions = ();
	type ExtensionsWeightInfo = weights::frame_system_extensions::WeightInfo<Runtime>;
}

impl pallet_msa::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_msa::weights::SubstrateWeight<Runtime>;
	// The conversion to a 32 byte AccountId
	type ConvertIntoAccountId32 = ConvertInto;
	// The maximum number of public keys per MSA
	type MaxPublicKeysPerMsa = MsaMaxPublicKeysPerMsa;
	// The maximum number of schema grants per delegation
	type MaxSchemaGrantsPerDelegation = MaxSchemaGrants;
	// The maximum provider name size (in bytes)
	type MaxProviderNameSize = MsaMaxProviderNameSize;
	// The type that provides schema related info
	type SchemaValidator = Schemas;
	// The type that provides `Handle` related info for a given `MessageSourceAccount`
	type HandleProvider = Handles;
	// The number of blocks per virtual bucket
	type MortalityWindowSize = MSAMortalityWindowSize;
	// The maximum number of signatures that can be stored in the payload signature registry
	type MaxSignaturesStored = MSAMaxSignaturesStored;
	// The proposal type
	type Proposal = RuntimeCall;
	// The Council proposal provider interface
	type ProposalProvider = CouncilProposalProvider;
	// The origin that is allowed to approve recovery providers
	#[cfg(any(feature = "frequency", feature = "runtime-benchmarks"))]
	type RecoveryProviderApprovalOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 2, 3>,
	>;
	#[cfg(not(any(feature = "frequency", feature = "runtime-benchmarks")))]
	type RecoveryProviderApprovalOrigin = EnsureSigned<AccountId>;
	// The origin that is allowed to create providers via governance
	type CreateProviderViaGovernanceOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureMembers<AccountId, CouncilCollective, 1>,
	>;
	// The Currency type for managing MSA token balances
	type Currency = Balances;
}

parameter_types! {
	/// The maximum number of eras over which one can claim rewards
	pub const ProviderBoostHistoryLimit : u32 = 30;
	/// The number of chunks of Reward Pool history we expect to store
	pub const RewardPoolChunkLength: u32 = 5;
}
// RewardPoolChunkLength MUST be a divisor of ProviderBoostHistoryLimit
const_assert!(ProviderBoostHistoryLimit::get() % RewardPoolChunkLength::get() == 0);

impl pallet_capacity::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_capacity::weights::SubstrateWeight<Runtime>;
	type Currency = Balances;
	type MinimumStakingAmount = CapacityMinimumStakingAmount;
	type MinimumTokenBalance = CapacityMinimumTokenBalance;
	type TargetValidator = Msa;
	type MaxUnlockingChunks = CapacityMaxUnlockingChunks;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = Msa;
	type UnstakingThawPeriod = CapacityUnstakingThawPeriod;
	type MaxEpochLength = CapacityMaxEpochLength;
	type EpochNumber = u32;
	type CapacityPerToken = CapacityPerToken;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type EraLength = CapacityRewardEraLength;
	type ProviderBoostHistoryLimit = ProviderBoostHistoryLimit;
	type RewardsProvider = Capacity;
	type MaxRetargetsPerRewardEra = ConstU32<2>;
	// Value determined by desired inflation rate limits for chosen economic model
	type RewardPoolPerEra = ConstU128<{ currency::CENTS.saturating_mul(153_424_650u128) }>;
	type RewardPercentCap = CapacityRewardCap;
	// Must evenly divide ProviderBoostHistoryLimit
	type RewardPoolChunkLength = RewardPoolChunkLength;
}

impl pallet_schemas::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_schemas::weights::SubstrateWeight<Runtime>;
	// The maximum number of intents that can be registered
	type MaxIntentRegistrations = IntentsMaxRegistrations;
	// The maximum number of intents that can belong to a single IntentGroup
	type MaxIntentsPerIntentGroup = IntentGroupMaxIntents;
	// The minimum size (in bytes) for a schema model
	type MinSchemaModelSizeBytes = SchemasMinModelSizeBytes;
	// The maximum number of schemas that can be registered
	type MaxSchemaRegistrations = SchemasMaxRegistrations;
	// The maximum length of a schema model (in bytes)
	type SchemaModelMaxBytesBoundedVecLimit = SchemasMaxBytesBoundedVecLimit;
	// The proposal type
	type Proposal = RuntimeCall;
	// The Council proposal provider interface
	type ProposalProvider = CouncilProposalProvider;
	// The origin that is allowed to create schemas via governance
	type CreateSchemaViaGovernanceOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>,
	>;
	// Maximum number of schema grants that are allowed per schema
	type MaxSchemaSettingsPerSchema = MaxSchemaSettingsPerSchema;
}

// One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
pub type DepositBase = ConstU128<{ currency::deposit(1, 88) }>;
// Additional storage item size of 32 bytes.
pub type DepositFactor = ConstU128<{ currency::deposit(0, 32) }>;
pub type MaxSignatories = ConstU32<100>;

// See https://paritytech.github.io/substrate/master/pallet_multisig/pallet/trait.Config.html for
// the descriptions of these configs.
impl pallet_multisig::Config for Runtime {
	type BlockNumberProvider = System;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type DepositBase = DepositBase;
	type DepositFactor = DepositFactor;
	type MaxSignatories = MaxSignatories;
	type WeightInfo = weights::pallet_multisig::SubstrateWeight<Runtime>;
}

impl cumulus_pallet_weight_reclaim::Config for Runtime {
	type WeightInfo = weights::cumulus_pallet_weight_reclaim::SubstrateWeight<Runtime>;
}

/// Need this declaration method for use + type safety in benchmarks
pub type MaxReleaseSchedules = ConstU32<{ MAX_RELEASE_SCHEDULES }>;

pub struct EnsureTimeReleaseOrigin;

impl EnsureOrigin<RuntimeOrigin> for EnsureTimeReleaseOrigin {
	type Success = AccountId;

	fn try_origin(o: RuntimeOrigin) -> Result<Self::Success, RuntimeOrigin> {
		match o.clone().into() {
			Ok(pallet_time_release::Origin::<Runtime>::TimeRelease(who)) => Ok(who),
			_ => Err(o),
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<RuntimeOrigin, ()> {
		Ok(RuntimeOrigin::root())
	}
}

// See https://paritytech.github.io/substrate/master/pallet_vesting/index.html for
// the descriptions of these configs.
impl pallet_time_release::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type Currency = Balances;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeHoldReason = RuntimeHoldReason;
	type MinReleaseTransfer = MinReleaseTransfer;
	type TransferOrigin = EnsureSigned<AccountId>;
	type WeightInfo = pallet_time_release::weights::SubstrateWeight<Runtime>;
	type MaxReleaseSchedules = MaxReleaseSchedules;
	#[cfg(any(not(feature = "frequency-no-relay"), feature = "frequency-lint-check"))]
	type BlockNumberProvider = RelaychainDataProvider<Runtime>;
	#[cfg(feature = "frequency-no-relay")]
	type BlockNumberProvider = System;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type SchedulerProvider = SchedulerProvider;
	type RuntimeCall = RuntimeCall;
	type TimeReleaseOrigin = EnsureTimeReleaseOrigin;
}

// See https://paritytech.github.io/substrate/master/pallet_timestamp/index.html for
// the descriptions of these configs.
impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	#[cfg(not(feature = "frequency-no-relay"))]
	type OnTimestampSet = Aura;
	#[cfg(feature = "frequency-no-relay")]
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = weights::pallet_timestamp::SubstrateWeight<Runtime>;
}

// See https://paritytech.github.io/substrate/master/pallet_authorship/index.html for
// the descriptions of these configs.
impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type EventHandler = (CollatorSelection,);
}

parameter_types! {
	pub const ExistentialDeposit: u128 = EXISTENTIAL_DEPOSIT;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = BalancesMaxLocks;
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = weights::pallet_balances::SubstrateWeight<Runtime>;
	type MaxReserves = BalancesMaxReserves;
	type ReserveIdentifier = [u8; 8];
	type MaxFreezes = BalancesMaxFreezes;
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type FreezeIdentifier = RuntimeFreezeReason;
	type DoneSlashHandler = ();
}
// Needs parameter_types! for the Weight type
parameter_types! {
	// The maximum weight that may be scheduled per block for any dispatchables of less priority than schedule::HARD_DEADLINE.
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(30) * RuntimeBlockWeights::get().max_block;
	pub MaxCollectivesProposalWeight: Weight = Perbill::from_percent(50) * RuntimeBlockWeights::get().max_block;
}

// See also https://docs.rs/pallet-scheduler/latest/pallet_scheduler/trait.Config.html
impl pallet_scheduler::Config for Runtime {
	type BlockNumberProvider = System;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type MaximumWeight = MaximumSchedulerWeight;
	/// Origin to schedule or cancel calls
	/// Set to Root or a simple majority of the Frequency Council
	type ScheduleOrigin = EitherOfDiverse<
		EitherOfDiverse<
			EnsureRoot<AccountId>,
			pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 2>,
		>,
		EnsureTimeReleaseOrigin,
	>;

	type MaxScheduledPerBlock = SchedulerMaxScheduledPerBlock;
	type WeightInfo = weights::pallet_scheduler::SubstrateWeight<Runtime>;
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type Preimages = Preimage;
}

parameter_types! {
	pub const PreimageHoldReason: RuntimeHoldReason = RuntimeHoldReason::Preimage(pallet_preimage::HoldReason::Preimage);
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

	type Consideration = HoldConsideration<
		AccountId,
		Balances,
		PreimageHoldReason,
		LinearStoragePrice<PreimageBaseDeposit, PreimageByteDeposit, Balance>,
	>;
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
	type WeightInfo = weights::pallet_collective_council::SubstrateWeight<Runtime>;
	type SetMembersOrigin = EnsureRoot<Self::AccountId>;
	type MaxProposalWeight = MaxCollectivesProposalWeight;
	type DisapproveOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 2, 3>,
	>;
	type KillOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 2, 3>,
	>;
	type Consideration = ();
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
	type WeightInfo = weights::pallet_collective_technical_committee::SubstrateWeight<Runtime>;
	type SetMembersOrigin = EnsureRoot<Self::AccountId>;
	type MaxProposalWeight = MaxCollectivesProposalWeight;
	type DisapproveOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCommitteeCollective, 2, 3>,
	>;
	type KillOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCommitteeCollective, 2, 3>,
	>;
	type Consideration = ();
}

// see https://paritytech.github.io/substrate/master/pallet_democracy/pallet/trait.Config.html
// for the definitions of these configs
impl pallet_democracy::Config for Runtime {
	type CooloffPeriod = CooloffPeriod;
	type Currency = Balances;
	type EnactmentPeriod = EnactmentPeriod;
	type RuntimeEvent = RuntimeEvent;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	type InstantAllowed = ConstBool<true>;
	type LaunchPeriod = LaunchPeriod;
	type MaxProposals = DemocracyMaxProposals;
	type MaxVotes = DemocracyMaxVotes;
	type MinimumDeposit = MinimumDeposit;
	type Scheduler = Scheduler;
	type Slash = ();
	// Treasury;
	type WeightInfo = weights::pallet_democracy::SubstrateWeight<Runtime>;
	type VoteLockingPeriod = EnactmentPeriod;
	// Same as EnactmentPeriod
	type VotingPeriod = VotingPeriod;
	type Preimages = Preimage;
	type MaxDeposits = ConstU32<100>;
	type MaxBlacklisted = ConstU32<100>;

	// See https://paritytech.github.io/substrate/master/pallet_democracy/index.html for
	// the descriptions of these origins.
	// See https://paritytech.github.io/substrate/master/pallet_democracy/pallet/trait.Config.html for
	// the definitions of these config traits.
	/// A unanimous council can have the next scheduled referendum be a straight default-carries
	/// (NTB) vote.
	type ExternalDefaultOrigin = EitherOfDiverse<
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 1>,
		frame_system::EnsureRoot<AccountId>,
	>;

	/// A simple-majority of 50% + 1 can have the next scheduled referendum be a straight majority-carries vote.
	type ExternalMajorityOrigin = EitherOfDiverse<
		pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>,
		frame_system::EnsureRoot<AccountId>,
	>;
	/// A straight majority (at least 50%) of the council can decide what their next motion is.
	type ExternalOrigin = EitherOfDiverse<
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 2>,
		frame_system::EnsureRoot<AccountId>,
	>;
	// Origin from which the new proposal can be made.
	// The success variant is the account id of the depositor.
	type SubmitOrigin = frame_system::EnsureSigned<AccountId>;

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

parameter_types! {
	pub TreasuryAccount: AccountId = TreasuryPalletId::get().into_account_truncating();
	pub const PayoutSpendPeriod: BlockNumber = 30 * DAYS;
	pub const MaxSpending : Balance = 100_000_000 * UNITS;
}

// See https://paritytech.github.io/substrate/master/pallet_treasury/index.html for
// the descriptions of these configs.
impl pallet_treasury::Config for Runtime {
	/// Treasury Account: 5EYCAe5ijiYfyeZ2JJCGq56LmPyNRAKzpG4QkoQkkQNB5e6Z
	type PalletId = TreasuryPalletId;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;

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
	#[cfg(not(feature = "runtime-benchmarks"))]
	type SpendOrigin = frame_support::traits::NeverEnsureOrigin<Balance>;
	#[cfg(feature = "runtime-benchmarks")]
	type SpendOrigin = MapSuccess<EnsureSigned<AccountId>, Replace<MaxSpending>>;

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

	type AssetKind = ();
	type Beneficiary = AccountId;
	type BeneficiaryLookup = IdentityLookup<Self::Beneficiary>;
	type Paymaster = PayFromAccount<Balances, TreasuryAccount>;
	type BalanceConverter = UnityAssetBalanceConversion;
	type PayoutPeriod = PayoutSpendPeriod;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

// See https://paritytech.github.io/substrate/master/pallet_transaction_payment/index.html for
// the descriptions of these configs.
impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = pallet_transaction_payment::FungibleAdapter<Balances, ()>;
	type WeightToFee = WeightToFee;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
	type OperationalFeeMultiplier = TransactionPaymentOperationalFeeMultiplier;
	type WeightInfo = weights::pallet_transaction_payment::SubstrateWeight<Runtime>;
}

use crate::ethereum::EthereumCompatibleAccountIdLookup;
use pallet_frequency_tx_payment::Call as FrequencyPaymentCall;
use pallet_handles::Call as HandlesCall;
use pallet_messages::Call as MessagesCall;
use pallet_msa::Call as MsaCall;
use pallet_stateful_storage::Call as StatefulStorageCall;

pub struct CapacityEligibleCalls;
impl GetStableWeight<RuntimeCall, Weight> for CapacityEligibleCalls {
	fn get_stable_weight(call: &RuntimeCall) -> Option<Weight> {
		use pallet_frequency_tx_payment::capacity_stable_weights::WeightInfo;
		match call {
            RuntimeCall::Msa(MsaCall::add_public_key_to_msa { .. }) => Some(
                capacity_stable_weights::SubstrateWeight::<Runtime>::add_public_key_to_msa()
            ),
            RuntimeCall::Msa(MsaCall::create_sponsored_account_with_delegation { add_provider_payload, .. }) => Some(capacity_stable_weights::SubstrateWeight::<Runtime>::create_sponsored_account_with_delegation(add_provider_payload.schema_ids.len() as u32)),
            RuntimeCall::Msa(MsaCall::grant_delegation { add_provider_payload, .. }) => Some(capacity_stable_weights::SubstrateWeight::<Runtime>::grant_delegation(add_provider_payload.schema_ids.len() as u32)),
            &RuntimeCall::Msa(MsaCall::add_recovery_commitment { .. }) => Some(
                capacity_stable_weights::SubstrateWeight::<Runtime>::add_recovery_commitment()
            ),
            &RuntimeCall::Msa(MsaCall::recover_account { .. }) => Some(
                capacity_stable_weights::SubstrateWeight::<Runtime>::recover_account()
            ),
            RuntimeCall::Messages(MessagesCall::add_ipfs_message { .. }) => Some(capacity_stable_weights::SubstrateWeight::<Runtime>::add_ipfs_message()),
            RuntimeCall::Messages(MessagesCall::add_onchain_message { payload, .. }) => Some(capacity_stable_weights::SubstrateWeight::<Runtime>::add_onchain_message(payload.len() as u32)),
            RuntimeCall::StatefulStorage(StatefulStorageCall::apply_item_actions { actions, .. }) => Some(capacity_stable_weights::SubstrateWeight::<Runtime>::apply_item_actions(StatefulStorage::sum_add_actions_bytes(actions))),
            RuntimeCall::StatefulStorage(StatefulStorageCall::upsert_page { payload, .. }) => Some(capacity_stable_weights::SubstrateWeight::<Runtime>::upsert_page(payload.len() as u32)),
            RuntimeCall::StatefulStorage(StatefulStorageCall::delete_page { .. }) => Some(capacity_stable_weights::SubstrateWeight::<Runtime>::delete_page()),
            RuntimeCall::StatefulStorage(StatefulStorageCall::apply_item_actions_with_signature_v2 { payload, .. }) => Some(capacity_stable_weights::SubstrateWeight::<Runtime>::apply_item_actions_with_signature(StatefulStorage::sum_add_actions_bytes(&payload.actions))),
            RuntimeCall::StatefulStorage(StatefulStorageCall::upsert_page_with_signature_v2 { payload, .. }) => Some(capacity_stable_weights::SubstrateWeight::<Runtime>::upsert_page_with_signature(payload.payload.len() as u32)),
            RuntimeCall::StatefulStorage(StatefulStorageCall::delete_page_with_signature_v2 { .. }) => Some(capacity_stable_weights::SubstrateWeight::<Runtime>::delete_page_with_signature()),
            RuntimeCall::Handles(HandlesCall::claim_handle { payload, .. }) => Some(capacity_stable_weights::SubstrateWeight::<Runtime>::claim_handle(payload.base_handle.len() as u32)),
            RuntimeCall::Handles(HandlesCall::change_handle { payload, .. }) => Some(capacity_stable_weights::SubstrateWeight::<Runtime>::change_handle(payload.base_handle.len() as u32)),
            _ => None,
        }
	}

	fn get_inner_calls(outer_call: &RuntimeCall) -> Option<Vec<&RuntimeCall>> {
		match outer_call {
			RuntimeCall::FrequencyTxPayment(FrequencyPaymentCall::pay_with_capacity {
				call,
				..
			}) => Some(vec![call]),
			RuntimeCall::FrequencyTxPayment(
				FrequencyPaymentCall::pay_with_capacity_batch_all { calls, .. },
			) => Some(calls.iter().collect()),
			_ => Some(vec![outer_call]),
		}
	}
}

impl pallet_frequency_tx_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Capacity = Capacity;
	type WeightInfo = pallet_frequency_tx_payment::weights::SubstrateWeight<Runtime>;
	type CapacityCalls = CapacityEligibleCalls;
	type OnChargeCapacityTransaction = pallet_frequency_tx_payment::CapacityAdapter<Balances, Msa>;
	type BatchProvider = CapacityBatchProvider;
	type MaximumCapacityBatchLength = MaximumCapacityBatchLength;
	type MsaKeyProvider = Msa;
	type MsaCallFilter = MsaCallFilter;
}

/// Configurations for passkey pallet
impl pallet_passkey::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type WeightInfo = pallet_passkey::weights::SubstrateWeight<Runtime>;
	type ConvertIntoAccountId32 = ConvertInto;
	type PasskeyCallFilter = PasskeyCallFilter;
	#[cfg(feature = "runtime-benchmarks")]
	type Currency = Balances;
}

#[cfg(any(not(feature = "frequency-no-relay"), feature = "frequency-lint-check"))]
/// Maximum number of blocks simultaneously accepted by the Runtime, not yet included
/// into the relay chain.
const UNINCLUDED_SEGMENT_CAPACITY: u32 = 3;

#[cfg(any(not(feature = "frequency-no-relay"), feature = "frequency-lint-check"))]
/// How many parachain blocks are processed by the relay chain per parent. Limits the
/// number of blocks authored per slot.
const BLOCK_PROCESSING_VELOCITY: u32 = 1;
#[cfg(any(not(feature = "frequency-no-relay"), feature = "frequency-lint-check"))]
/// Relay chain slot duration, in milliseconds.
const RELAY_CHAIN_SLOT_DURATION_MILLIS: u32 = 6_000;

// See https://paritytech.github.io/substrate/master/pallet_parachain_system/index.html for
// the descriptions of these configs.
#[cfg(any(
	not(feature = "frequency-no-relay"),
	feature = "frequency-lint-check",
	feature = "frequency-bridging"
))]
impl cumulus_pallet_parachain_system::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnSystemEvent = ();
	type SelfParaId = parachain_info::Pallet<Runtime>;

	#[cfg(feature = "frequency-bridging")]
	type DmpQueue = frame_support::traits::EnqueueWithOrigin<MessageQueue, RelayOrigin>;

	#[cfg(not(feature = "frequency-bridging"))]
	type DmpQueue = frame_support::traits::EnqueueWithOrigin<(), sp_core::ConstU8<0>>;

	#[cfg(not(feature = "frequency-bridging"))]
	type ReservedDmpWeight = ();

	#[cfg(feature = "frequency-bridging")]
	type ReservedDmpWeight = ReservedDmpWeight;

	#[cfg(not(feature = "frequency-bridging"))]
	type OutboundXcmpMessageSource = ();

	#[cfg(feature = "frequency-bridging")]
	type OutboundXcmpMessageSource = XcmpQueue;

	#[cfg(not(feature = "frequency-bridging"))]
	type XcmpMessageHandler = ();

	#[cfg(feature = "frequency-bridging")]
	type XcmpMessageHandler = XcmpQueue;

	#[cfg(not(feature = "frequency-bridging"))]
	type ReservedXcmpWeight = ();

	#[cfg(feature = "frequency-bridging")]
	type ReservedXcmpWeight = ReservedXcmpWeight;

	type CheckAssociatedRelayNumber = RelayNumberMonotonicallyIncreases;
	type WeightInfo = ();
	type ConsensusHook = ConsensusHook;
	type SelectCore = DefaultCoreSelector<Runtime>;
}

#[cfg(any(not(feature = "frequency-no-relay"), feature = "frequency-lint-check"))]
pub type ConsensusHook = cumulus_pallet_aura_ext::FixedVelocityConsensusHook<
	Runtime,
	RELAY_CHAIN_SLOT_DURATION_MILLIS,
	BLOCK_PROCESSING_VELOCITY,
	UNINCLUDED_SEGMENT_CAPACITY,
>;

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
	type DisablingStrategy = ();
	type WeightInfo = weights::pallet_session::SubstrateWeight<Runtime>;
}

// See https://paritytech.github.io/substrate/master/pallet_aura/index.html for
// the descriptions of these configs.
impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = AuraMaxAuthorities;
	type AllowMultipleBlocksPerSlot = ConstBool<true>;
	type SlotDuration = ConstU64<SLOT_DURATION>;
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
	type MinEligibleCollators = CollatorMinCandidates;

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

	type WeightInfo = weights::pallet_collator_selection::SubstrateWeight<Runtime>;
}

// https://paritytech.github.io/polkadot-sdk/master/pallet_proxy/pallet/trait.Config.html
impl pallet_proxy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type MaxProxies = MaxProxies;
	type MaxPending = MaxPending;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
	type WeightInfo = weights::pallet_proxy::SubstrateWeight<Runtime>;
	type BlockNumberProvider = System;
}

// End Proxy Pallet Config

impl pallet_messages::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_messages::weights::SubstrateWeight<Runtime>;
	// The type that supplies MSA info
	type MsaInfoProvider = Msa;
	// The type that validates schema grants
	type SchemaGrantValidator = Msa;
	// The type that provides schema info
	type SchemaProvider = Schemas;
	// The maximum message payload in bytes
	type MessagesMaxPayloadSizeBytes = MessagesMaxPayloadSizeBytes;

	/// A set of helper functions for benchmarking.
	#[cfg(feature = "runtime-benchmarks")]
	type MsaBenchmarkHelper = Msa;
	#[cfg(feature = "runtime-benchmarks")]
	type SchemaBenchmarkHelper = Schemas;
}

impl pallet_stateful_storage::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_stateful_storage::weights::SubstrateWeight<Runtime>;
	/// The maximum size of a page (in bytes) for an Itemized storage model
	type MaxItemizedPageSizeBytes = MaxItemizedPageSizeBytes;
	/// The maximum size of a page (in bytes) for a Paginated storage model
	type MaxPaginatedPageSizeBytes = MaxPaginatedPageSizeBytes;
	/// The maximum size of a single item in an itemized storage model (in bytes)
	type MaxItemizedBlobSizeBytes = MaxItemizedBlobSizeBytes;
	/// The maximum number of pages in a Paginated storage model
	type MaxPaginatedPageId = MaxPaginatedPageId;
	/// The maximum number of actions in itemized actions
	type MaxItemizedActionsCount = MaxItemizedActionsCount;
	/// The type that supplies MSA info
	type MsaInfoProvider = Msa;
	/// The type that validates schema grants
	type SchemaGrantValidator = Msa;
	/// The type that provides schema info
	type SchemaProvider = Schemas;
	/// Hasher for Child Tree keys
	type KeyHasher = Twox128;
	/// The conversion to a 32 byte AccountId
	type ConvertIntoAccountId32 = ConvertInto;
	/// The number of blocks per virtual bucket
	type MortalityWindowSize = StatefulMortalityWindowSize;

	/// A set of helper functions for benchmarking.
	#[cfg(feature = "runtime-benchmarks")]
	type MsaBenchmarkHelper = Msa;
	#[cfg(feature = "runtime-benchmarks")]
	type SchemaBenchmarkHelper = Schemas;
}

impl pallet_handles::Config for Runtime {
	/// The overarching event type.
	type RuntimeEvent = RuntimeEvent;
	/// Weight information for extrinsics in this pallet.
	type WeightInfo = pallet_handles::weights::SubstrateWeight<Runtime>;
	/// The type that supplies MSA info
	type MsaInfoProvider = Msa;
	/// The minimum suffix value
	type HandleSuffixMin = HandleSuffixMin;
	/// The maximum suffix value
	type HandleSuffixMax = HandleSuffixMax;
	/// The conversion to a 32 byte AccountId
	type ConvertIntoAccountId32 = ConvertInto;
	// The number of blocks per virtual bucket
	type MortalityWindowSize = MSAMortalityWindowSize;
	/// A set of helper functions for benchmarking.
	#[cfg(feature = "runtime-benchmarks")]
	type MsaBenchmarkHelper = Msa;
}

// ---------- Foreign Assets pallet configuration ----------
#[cfg(feature = "frequency-bridging")]
impl pallet_assets::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type AssetId = ForeignAssetsAssetId;
	type AssetIdParameter = ForeignAssetsAssetId;
	type Currency = Balances;

	type CreateOrigin = AsEnsureOriginWithArg<EnsureNever<AccountId>>;
	type ForceOrigin = EnsureRoot<AccountId>;

	type AssetDeposit = ForeignAssetsAssetDeposit;
	type MetadataDepositBase = ForeignAssetsMetadataDepositBase;
	type MetadataDepositPerByte = ForeignAssetsMetadataDepositPerByte;
	type ApprovalDeposit = ForeignAssetsApprovalDeposit;
	type StringLimit = ForeignAssetsAssetsStringLimit;

	type Freezer = ();
	type Extra = ();
	type WeightInfo = pallet_assets::weights::SubstrateWeight<Runtime>;
	type CallbackHandle = ();
	type AssetAccountDeposit = ForeignAssetsAssetAccountDeposit;
	type RemoveItemsLimit = frame_support::traits::ConstU32<1000>;

	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = xcm::xcm_config::XcmBenchmarkHelper;
	type Holder = ();
}

// See https://paritytech.github.io/substrate/master/pallet_sudo/index.html for
// the descriptions of these configs.
#[cfg(any(not(feature = "frequency"), feature = "frequency-lint-check"))]
impl pallet_sudo::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	/// using original weights from sudo pallet
	type WeightInfo = pallet_sudo::weights::SubstrateWeight<Runtime>;
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
	pub enum Runtime {
		// System support stuff.
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>} = 0,
		#[cfg(any(
			not(feature = "frequency-no-relay"),
			feature = "frequency-lint-check",
			feature = "frequency-bridging"
		))]
		ParachainSystem: cumulus_pallet_parachain_system::{ Pallet, Call, Config<T>, Storage, Inherent, Event<T> } = 1,
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent} = 2,
		ParachainInfo: parachain_info::{Pallet, Storage, Config<T>} = 3,

		// Sudo removed from mainnet Jan 2023
		#[cfg(any(not(feature = "frequency"), feature = "frequency-lint-check"))]
		Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T> }= 4,

		Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>, HoldReason} = 5,
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
		Treasury: pallet_treasury::{Pallet, Call, Storage, Config<T>, Event<T>} = 14,

		// Collator support. The order of these 4 are important and shall not change.
		Authorship: pallet_authorship::{Pallet, Storage} = 20,
		CollatorSelection: pallet_collator_selection::{Pallet, Call, Storage, Event<T>, Config<T>} = 21,
		Session: pallet_session::{Pallet, Call, Storage, Event<T>, Config<T>} = 22,
		Aura: pallet_aura::{Pallet, Storage, Config<T>} = 23,
		AuraExt: cumulus_pallet_aura_ext::{Pallet, Storage, Config<T>} = 24,

		// Signatures
		Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>} = 30,

		// FRQCY Update
		TimeRelease: pallet_time_release::{Pallet, Call, Storage, Event<T>, Config<T>, Origin<T>, FreezeReason, HoldReason} = 40,

		// Allowing accounts to give permission to other accounts to dispatch types of calls from their signed origin
		Proxy: pallet_proxy = 43,

		// Substrate weights
		WeightReclaim: cumulus_pallet_weight_reclaim::{Pallet, Storage} = 50,

		// Frequency related pallets
		Msa: pallet_msa::{Pallet, Call, Storage, Event<T>} = 60,
		Messages: pallet_messages::{Pallet, Call, Storage, Event<T>} = 61,
		Schemas: pallet_schemas::{Pallet, Call, Storage, Event<T>, Config<T>} = 62,
		StatefulStorage: pallet_stateful_storage::{Pallet, Call, Storage, Event<T>} = 63,
		Capacity: pallet_capacity::{Pallet, Call, Storage, Event<T>, FreezeReason} = 64,
		FrequencyTxPayment: pallet_frequency_tx_payment::{Pallet, Call, Event<T>} = 65,
		Handles: pallet_handles::{Pallet, Call, Storage, Event<T>} = 66,
		Passkey: pallet_passkey::{Pallet, Call, Storage, Event<T>, ValidateUnsigned} = 67,

		#[cfg(feature = "frequency-bridging")]
		XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>} = 71,

		#[cfg(feature = "frequency-bridging")]
		PolkadotXcm: pallet_xcm::{Pallet, Call, Storage, Event<T>, Origin } = 72,

		#[cfg(feature = "frequency-bridging")]
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin} = 73,

		#[cfg(feature = "frequency-bridging")]
		MessageQueue: pallet_message_queue::{Pallet, Call, Storage, Event<T>} = 74,

		#[cfg(feature = "frequency-bridging")]
		ForeignAssets: pallet_assets::{Pallet, Call, Storage, Event<T>} = 75,
	}
);

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	define_benchmarks!(
		// Substrate
		[frame_system, SystemBench::<Runtime>]
		[frame_system_extensions, SystemExtensionsBench::<Runtime>]
		[cumulus_pallet_weight_reclaim, WeightReclaim]
		[pallet_assets, ForeignAssets]
		[pallet_balances, Balances]
		[pallet_collective, Council]
		[pallet_collective, TechnicalCommittee]
		[pallet_preimage, Preimage]
		[pallet_democracy, Democracy]
		[pallet_scheduler, Scheduler]
		[pallet_session, SessionBench::<Runtime>]
		[pallet_timestamp, Timestamp]
		[pallet_collator_selection, CollatorSelection]
		[pallet_multisig, Multisig]
		[pallet_utility, Utility]
		[pallet_proxy, Proxy]
		[pallet_transaction_payment, TransactionPayment]
		[cumulus_pallet_xcmp_queue, XcmpQueue]
		[pallet_message_queue, MessageQueue]

		// Frequency
		[pallet_msa, Msa]
		[pallet_schemas, Schemas]
		[pallet_messages, Messages]
		[pallet_stateful_storage, StatefulStorage]
		[pallet_handles, Handles]
		[pallet_time_release, TimeRelease]
		[pallet_treasury, Treasury]
		[pallet_capacity, Capacity]
		[pallet_frequency_tx_payment, FrequencyTxPayment]
		[pallet_passkey, Passkey]

		[pallet_xcm_benchmarks::fungible, XcmBalances]
		[pallet_xcm_benchmarks::generic, XcmGeneric]
	);
}

#[cfg(any(
	not(feature = "frequency-no-relay"),
	feature = "frequency-lint-check",
	feature = "frequency-bridging"
))]
cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
}

// The implementation has to be here due to the linking in the macro.
// It CANNOT be extracted into a separate file
sp_api::impl_runtime_apis! {
	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(SLOT_DURATION)
		}

		fn authorities() -> Vec<AuraId> {
			pallet_aura::Authorities::<Runtime>::get().into_inner()
		}
	}

	#[cfg(any(not(feature = "frequency-no-relay"), feature = "frequency-lint-check"))]
	impl cumulus_primitives_aura::AuraUnincludedSegmentApi<Block> for Runtime {
		fn can_build_upon(
			included_hash: <Block as BlockT>::Hash,
			slot: cumulus_primitives_aura::Slot,
		) -> bool {
			ConsensusHook::can_build_upon(included_hash, slot)
		}
	}

	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) -> sp_runtime::ExtrinsicInclusionMode {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}

		fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}

		fn metadata_versions() -> Vec<u32> {
			Runtime::metadata_versions()
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

	impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
		fn build_state(config: Vec<u8>) -> sp_genesis_builder::Result {
			build_state::<RuntimeGenesisConfig>(config)
		}

		fn get_preset(id: &Option<sp_genesis_builder::PresetId>) -> Option<Vec<u8>> {
			get_preset::<RuntimeGenesisConfig>(id,  &crate::genesis::presets::get_preset)
		}

		fn preset_names() -> Vec<sp_genesis_builder::PresetId> {
			let mut presets = vec![];

			#[cfg(any(
				feature = "frequency-no-relay",
				feature = "frequency-local",
				feature = "frequency-lint-check"
			))]
			presets.extend(
			vec![
				sp_genesis_builder::PresetId::from("development"),
				sp_genesis_builder::PresetId::from("frequency-local"),
				sp_genesis_builder::PresetId::from("frequency"),
				sp_genesis_builder::PresetId::from("frequency-westend-local"),
			]);


			#[cfg(feature = "frequency-testnet")]
			presets.push(sp_genesis_builder::PresetId::from("frequency-testnet"));

			#[cfg(feature = "frequency-westend")]
			presets.push(sp_genesis_builder::PresetId::from("frequency-westend"));

			#[cfg(feature = "frequency")]
			presets.push(sp_genesis_builder::PresetId::from("frequency"));

			presets
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
		fn account_nonce(account: AccountId) -> Index {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
	// THIS QUERY_INFO IS FAILING AFTER THE CHANGES I MADE.
	// TO TEST: DID THIS ACTUALLY WORK ON LOCAL BEFORE THE CHANGES?
	// ERROR: `Bad input data provided to query_info: Codec error`
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
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(len: u32) -> Balance {
			TransactionPayment::length_to_fee(len)
		}
	}

	impl pallet_frequency_tx_payment_runtime_api::CapacityTransactionPaymentRuntimeApi<Block, Balance> for Runtime {
		fn compute_capacity_fee(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) ->pallet_transaction_payment::FeeDetails<Balance> {

			// if the call is wrapped in a batch, we need to get the weight of the outer call
			// and use that to compute the fee with the inner call's stable weight(s)
			let dispatch_weight = match &uxt.function {
				RuntimeCall::FrequencyTxPayment(pallet_frequency_tx_payment::Call::pay_with_capacity { .. }) |
				RuntimeCall::FrequencyTxPayment(pallet_frequency_tx_payment::Call::pay_with_capacity_batch_all { .. }) => {
					<<Block as BlockT>::Extrinsic as GetDispatchInfo>::get_dispatch_info(&uxt).call_weight
				},
				_ => {
					Weight::zero()
				}
			};
			FrequencyTxPayment::compute_capacity_fee_details(&uxt.function, &dispatch_weight, len)
		}
	}

	#[cfg(any(not(feature = "frequency-no-relay"), feature = "frequency-lint-check"))]
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

	#[api_version(3)]
	impl pallet_schemas_runtime_api::SchemasRuntimeApi<Block> for Runtime {
		fn get_by_schema_id(schema_id: SchemaId) -> Option<SchemaResponse> {
			Schemas::get_schema_by_id(schema_id)
		}

		fn get_schema_versions_by_name(schema_name: Vec<u8>) -> Option<Vec<SchemaVersionResponse>> {
			Schemas::get_schema_versions(schema_name)
		}

		fn get_registered_entities_by_name(name: Vec<u8>) -> Option<Vec<NameLookupResponse>> {
			Schemas::get_intent_or_group_ids_by_name(name)
		}
	}

	impl system_runtime_api::AdditionalRuntimeApi<Block> for Runtime {
		fn get_events() -> Vec<RpcEvent> {
			System::read_events_no_consensus().map(|e| (*e).into()).collect()
		}
	}

	impl pallet_msa_runtime_api::MsaRuntimeApi<Block, AccountId> for Runtime {
		fn has_delegation(delegator: DelegatorId, provider: ProviderId, block_number: BlockNumber, schema_id: Option<SchemaId>) -> bool {
			match schema_id {
				Some(sid) => Msa::ensure_valid_schema_grant(provider, delegator, sid, block_number).is_ok(),
				None => Msa::ensure_valid_delegation(provider, delegator, Some(block_number)).is_ok(),
			}
		}

		fn get_granted_schemas_by_msa_id(delegator: DelegatorId, provider: ProviderId) -> Option<Vec<SchemaGrant<SchemaId, BlockNumber>>> {
			match Msa::get_granted_schemas_by_msa_id(delegator, Some(provider)) {
				Ok(res) => match res.into_iter().next() {
					Some(delegation) => Some(delegation.permissions),
					None => None,
				},
				_ => None,
			}
		}

		fn get_all_granted_delegations_by_msa_id(delegator: DelegatorId) -> Vec<DelegationResponse<SchemaId, BlockNumber>> {
			Msa::get_granted_schemas_by_msa_id(delegator, None).unwrap_or_default()
		}

		fn get_ethereum_address_for_msa_id(msa_id: MessageSourceId) -> AccountId20Response {
			let account_id = Msa::msa_id_to_eth_address(msa_id);
			let account_id_checksummed = Msa::eth_address_to_checksummed_string(&account_id);
			AccountId20Response { account_id, account_id_checksummed }
		}

		fn validate_eth_address_for_msa(address: &H160, msa_id: MessageSourceId) -> bool {
			Msa::validate_eth_address_for_msa(address, msa_id)
		}
	}

	impl pallet_stateful_storage_runtime_api::StatefulStorageRuntimeApi<Block> for Runtime {
		fn get_paginated_storage(msa_id: MessageSourceId, schema_id: SchemaId) -> Result<Vec<PaginatedStorageResponse>, DispatchError> {
			StatefulStorage::get_paginated_storage(msa_id, schema_id)
		}

		fn get_itemized_storage(msa_id: MessageSourceId, schema_id: SchemaId) -> Result<ItemizedStoragePageResponse, DispatchError> {
			StatefulStorage::get_itemized_storage(msa_id, schema_id)
		}
	}

	#[api_version(3)]
	impl pallet_handles_runtime_api::HandlesRuntimeApi<Block> for Runtime {
		fn get_handle_for_msa(msa_id: MessageSourceId) -> Option<HandleResponse> {
			Handles::get_handle_for_msa(msa_id)
		}

		fn get_next_suffixes(base_handle: BaseHandle, count: u16) -> PresumptiveSuffixesResponse {
			Handles::get_next_suffixes(base_handle, count)
		}

		fn get_msa_for_handle(display_handle: DisplayHandle) -> Option<MessageSourceId> {
			Handles::get_msa_id_for_handle(display_handle)
		}
		fn validate_handle(base_handle: BaseHandle) -> bool {
			Handles::validate_handle(base_handle.to_vec())
		}
		fn check_handle(base_handle: BaseHandle) -> CheckHandleResponse {
			Handles::check_handle(base_handle.to_vec())
		}
	}

	impl pallet_capacity_runtime_api::CapacityRuntimeApi<Block, AccountId, Balance, BlockNumber> for Runtime {
		fn list_unclaimed_rewards(who: AccountId) -> Vec<UnclaimedRewardInfo<Balance, BlockNumber>> {
			match Capacity::list_unclaimed_rewards(&who) {
				Ok(rewards) => rewards.into_inner(),
				Err(_) => Vec::new(),
			}
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade(checks: UpgradeCheckSelect) -> (Weight, Weight) {
			log::info!("try-runtime::on_runtime_upgrade frequency.");
			let weight = Executive::try_runtime_upgrade(checks).unwrap();
			(weight, RuntimeBlockWeights::get().max_block)
		}

		fn execute_block(block: Block,
						state_root_check: bool,
						signature_check: bool,
						try_state: TryStateSelect,
		) -> Weight {
			log::info!(
				target: "runtime::frequency", "try-runtime: executing block #{} ({:?}) / root checks: {:?} / sanity-checks: {:?}",
				block.header.number,
				block.header.hash(),
				state_root_check,
				try_state,
			);
			Executive::try_execute_block(block, state_root_check, signature_check, try_state).expect("try_execute_block failed")
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{BenchmarkList};
			use frame_support::traits::StorageInfoTrait;
			use frame_system_benchmarking::Pallet as SystemBench;
			use frame_system_benchmarking::extensions::Pallet as SystemExtensionsBench;
			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;

			// This is defined once again in dispatch_benchmark, because list_benchmarks!
			// and add_benchmarks! are macros exported by define_benchmarks! macros and those types
			// are referenced in that call.
			type XcmBalances = pallet_xcm_benchmarks::fungible::Pallet::<Runtime>;
			type XcmGeneric = pallet_xcm_benchmarks::generic::Pallet::<Runtime>;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();
			(list, storage_info)
		}

		#[allow(deprecated, non_local_definitions)]
		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{BenchmarkBatch, BenchmarkError};

			use frame_system_benchmarking::Pallet as SystemBench;
			impl frame_system_benchmarking::Config for Runtime {}

			use frame_system_benchmarking::extensions::Pallet as SystemExtensionsBench;

			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;
			impl cumulus_pallet_session_benchmarking::Config for Runtime {}

			use frame_support::traits::{WhitelistedStorageKeys, TrackedStorageKey};
			let whitelist: Vec<TrackedStorageKey> = AllPalletsWithSystem::whitelisted_storage_keys();

			#[cfg(feature = "frequency-bridging")]
			impl pallet_xcm_benchmarks::Config for Runtime {
				type XcmConfig = xcm::xcm_config::XcmConfig;
				type AccountIdConverter = xcm::LocationToAccountId;
				type DeliveryHelper = xcm::benchmarks::ParachainDeliveryHelper;

				fn valid_destination() -> Result<xcm::benchmarks::Location, BenchmarkError> {
					xcm::benchmarks::create_foreign_asset_dot_on_frequency();
					Ok(xcm::benchmarks::AssetHubParachainLocation::get())
				}

				fn worst_case_holding(_depositable_count: u32) -> xcm::benchmarks::Assets {
					let mut assets = xcm::benchmarks::Assets::new();
					assets.push(xcm::benchmarks::Asset { id: xcm::benchmarks::AssetId(xcm::benchmarks::HereLocation::get()), fun: xcm::benchmarks::Fungibility::Fungible(u128::MAX) });
					assets.push(xcm::benchmarks::Asset { id: xcm::benchmarks::RelayAssetId::get(), fun: xcm::benchmarks::Fungibility::Fungible(u128::MAX / 2) });
					assets
				}
			}

			#[cfg(feature = "frequency-bridging")]
			impl pallet_xcm_benchmarks::fungible::Config for Runtime {
				type TransactAsset = Balances;
				type CheckedAccount = xcm::benchmarks::CheckAccount;
				type TrustedTeleporter = xcm::benchmarks::TrustedTeleporter;
				type TrustedReserve = xcm::benchmarks::TrustedReserve;

				fn get_asset() -> xcm::benchmarks::Asset {
					xcm::benchmarks::create_foreign_asset_dot_on_frequency();
					xcm::benchmarks::RelayAsset::get()
				}
			}

			#[cfg(feature = "frequency-bridging")]
			impl pallet_xcm_benchmarks::generic::Config for Runtime {
				type RuntimeCall = RuntimeCall;
				type TransactAsset = Balances;

				fn worst_case_response() -> (u64, xcm::benchmarks::Response) {
					(0u64, xcm::benchmarks::Response::Version(Default::default()))
				}

				// We do not support asset exchange on frequency
				fn worst_case_asset_exchange() -> Result<(xcm::benchmarks::Assets, xcm::benchmarks::Assets), BenchmarkError> {
					Err(BenchmarkError::Skip)
				}

				// We do not support universal origin permissioning.
				fn universal_alias() -> Result<(xcm::benchmarks::Location, xcm::benchmarks::Junction), BenchmarkError> {
					Err(BenchmarkError::Skip)
				}

				// We do not support transact instructions on frequency
				// But this helper also used to benchmark unsubscribe_version which we do support.
				fn transact_origin_and_runtime_call() -> Result<(xcm::benchmarks::Location, RuntimeCall), BenchmarkError> {
					Ok((xcm::benchmarks::RelayLocation::get(), frame_system::Call::remark_with_event { remark: vec![] }.into()))
				}

				fn subscribe_origin() -> Result<xcm::benchmarks::Location, BenchmarkError> {
					Ok(xcm::benchmarks::RelayLocation::get())
				}

				fn claimable_asset() -> Result<(xcm::benchmarks::Location, xcm::benchmarks::Location, xcm::benchmarks::Assets), BenchmarkError> {
					let origin = xcm::benchmarks::AssetHubParachainLocation::get();
					let assets = xcm::benchmarks::RelayAsset::get().into();
					let ticket = xcm::benchmarks::HereLocation::get();
					Ok((origin, ticket, assets))
				}

				fn fee_asset() -> Result<xcm::benchmarks::Asset, BenchmarkError> {
					Ok(xcm::benchmarks::RelayAsset::get())
				}

				// We do not support locking and unlocking on Frequency
				fn unlockable_asset() -> Result<(xcm::benchmarks::Location, xcm::benchmarks::Location, xcm::benchmarks::Asset), BenchmarkError> {
					Err(BenchmarkError::Skip)
				}

				// We do not support export message on Frequency
				fn export_message_origin_and_destination() -> Result<(xcm::benchmarks::Location, xcm::benchmarks::NetworkId, xcm::benchmarks::InteriorLocation), BenchmarkError> {
					Err(BenchmarkError::Skip)
				}

				// We do not support alias origin on Frequency
				fn alias_origin() -> Result<(xcm::benchmarks::Location, xcm::benchmarks::Location), BenchmarkError> {
					Err(BenchmarkError::Skip)
				}
			}

			#[cfg(feature = "frequency-bridging")]
			type XcmBalances = pallet_xcm_benchmarks::fungible::Pallet::<Runtime>;
			#[cfg(feature = "frequency-bridging")]
			type XcmGeneric = pallet_xcm_benchmarks::generic::Pallet::<Runtime>;


			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);
			add_benchmarks!(params, batches);

			Ok(batches)
		}


	}

	#[cfg(feature = "frequency-bridging")]
	impl xcm_runtime_apis::fees::XcmPaymentApi<Block> for Runtime {
		fn query_acceptable_payment_assets(xcm_version: staging_xcm::Version) -> Result<Vec<VersionedAssetId>, XcmPaymentApiError> {
			let acceptable_assets = vec![AssetLocationId(RelayLocation::get())];
			PolkadotXcm::query_acceptable_payment_assets(xcm_version, acceptable_assets)
		}

		// Frequency implementation of the query_weight_to_asset_fee function
		fn query_weight_to_asset_fee(weight: Weight, asset: VersionedAssetId) -> Result<u128, XcmPaymentApiError> {
			use frame_support::weights::WeightToFee;

			match asset.try_as::<AssetLocationId>() {
				Ok(asset_id) if asset_id.0 == NativeToken::get().0 => {
					// FRQCY/XRQCY, native token
					Ok(common_runtime::fee::WeightToFee::weight_to_fee(&weight))
				},
				Ok(asset_id) if asset_id.0 == RelayLocation::get() => {
					// DOT, WND, or KSM on the relay chain
					// calculate fee in DOT using Polkadot relay fee schedule
					let dot_fee = crate::polkadot_xcm_fee::default_fee_per_second()
						.saturating_mul(weight.ref_time() as u128)
						.saturating_div(WEIGHT_REF_TIME_PER_SECOND as u128);
					Ok(dot_fee)
				},
				Ok(asset_id) => {
					log::trace!(target: "xcm::xcm_runtime_apis", "query_weight_to_asset_fee - unhandled asset_id: {asset_id:?}!");
					Err(XcmPaymentApiError::AssetNotFound)
				},
				Err(_) => {
					log::trace!(target: "xcm::xcm_runtime_apis", "query_weight_to_asset_fee - failed to convert asset: {asset:?}!");
					Err(XcmPaymentApiError::VersionedConversionFailed)
				}
			}
		}

		fn query_xcm_weight(message: VersionedXcm<()>) -> Result<Weight, XcmPaymentApiError> {
			PolkadotXcm::query_xcm_weight(message)
		}

		fn query_delivery_fees(destination: VersionedLocation, message: VersionedXcm<()>) -> Result<VersionedAssets, XcmPaymentApiError> {
			PolkadotXcm::query_delivery_fees(destination, message)
		}
	}

	#[cfg(feature = "frequency-bridging")]
	impl xcm_runtime_apis::dry_run::DryRunApi<Block, RuntimeCall, RuntimeEvent, OriginCaller> for Runtime {
		fn dry_run_call(origin: OriginCaller, call: RuntimeCall, result_xcms_version: XcmVersion) -> Result<CallDryRunEffects<RuntimeEvent>, XcmDryRunApiError> {
			PolkadotXcm::dry_run_call::<Runtime, XcmRouter, OriginCaller, RuntimeCall>(origin, call, result_xcms_version)
		}

		fn dry_run_xcm(origin_location: VersionedLocation, xcm: VersionedXcm<RuntimeCall>) -> Result<XcmDryRunEffects<RuntimeEvent>, XcmDryRunApiError> {
			PolkadotXcm::dry_run_xcm::<Runtime, XcmRouter, RuntimeCall, XcmConfig>(origin_location, xcm)
		}
	}

	#[cfg(feature = "frequency-bridging")]
	impl xcm_runtime_apis::conversions::LocationToAccountApi<Block, AccountId> for Runtime {
		fn convert_location(location: VersionedLocation) -> Result<
			AccountId,
			xcm_runtime_apis::conversions::Error
		> {
			xcm_runtime_apis::conversions::LocationToAccountHelper::<
				AccountId,
				LocationToAccountId,
			>::convert_location(location)
		}
	}

	#[cfg(feature = "frequency-bridging")]
	impl xcm_runtime_apis::trusted_query::TrustedQueryApi<Block> for Runtime {
		fn is_trusted_reserve(asset: VersionedAsset, location: VersionedLocation) -> xcm_runtime_apis::trusted_query::XcmTrustedQueryResult {
			PolkadotXcm::is_trusted_reserve(asset, location)
		}
		fn is_trusted_teleporter(asset: VersionedAsset, location: VersionedLocation) -> xcm_runtime_apis::trusted_query::XcmTrustedQueryResult {
			PolkadotXcm::is_trusted_teleporter(asset, location)
		}
	}

	#[cfg(feature = "frequency-bridging")]
	impl xcm_runtime_apis::authorized_aliases::AuthorizedAliasersApi<Block> for Runtime {
		fn authorized_aliasers(target: VersionedLocation) -> Result<
			Vec<xcm_runtime_apis::authorized_aliases::OriginAliaser>,
			xcm_runtime_apis::authorized_aliases::Error
		> {
			PolkadotXcm::authorized_aliasers(target)
		}
		fn is_authorized_alias(origin: VersionedLocation, target: VersionedLocation) -> Result<
			bool,
			xcm_runtime_apis::authorized_aliases::Error
		> {
			PolkadotXcm::is_authorized_alias(origin, target)
		}
	}
}
