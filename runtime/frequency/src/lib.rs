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

use alloc::borrow::Cow;
use common_runtime::constants::currency::UNITS;
use core::marker::PhantomData;
#[cfg(any(not(feature = "frequency-no-relay"), feature = "frequency-lint-check"))]
use cumulus_pallet_parachain_system::{
	DefaultCoreSelector, RelayNumberMonotonicallyIncreases, RelaychainDataProvider,
};
#[cfg(any(feature = "runtime-benchmarks", feature = "test"))]
use frame_support::traits::MapSuccess;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata, H256};
#[cfg(any(feature = "runtime-benchmarks", feature = "test"))]
use sp_runtime::traits::Replace;
use sp_runtime::{
	generic, impl_opaque_keys,
	traits::{
		AccountIdConversion, BlakeTwo256, Block as BlockT, ConvertInto, DispatchInfoOf,
		Dispatchable, IdentityLookup, PostDispatchInfoOf, UniqueSaturatedInto,
	},
	transaction_validity::{TransactionSource, TransactionValidity, TransactionValidityError},
	ApplyExtrinsicResult, DispatchError,
};

use pallet_collective::Members;

#[cfg(any(feature = "runtime-benchmarks", feature = "test"))]
use pallet_collective::ProposalCount;

use parity_scale_codec::{Decode, Encode};

#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use static_assertions::const_assert;

use common_primitives::signatures::{AccountAddressMapper, EthereumAddressMapper};
use fp_evm::weight_per_gas;
use pallet_evm::{
	Account as EVMAccount, AddressMapping, EnsureAddressTruncated, FeeCalculator, Runner,
};
use sp_core::U256;

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

use fp_rpc::TransactionStatus;
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
		EqualPrivilegeOnly, GetStorageVersion, InstanceFilter, LinearStoragePrice, OnFinalize,
		OnRuntimeUpgrade,
	},
	weights::{ConstantMultiplier, Weight},
	Twox128,
};
use frame_system::{
	limits::{BlockLength, BlockWeights},
	EnsureRoot, EnsureSigned,
};
use pallet_ethereum::{Call::transact, PostLogContent, Transaction as EthereumTransaction};

use alloc::{boxed::Box, vec, vec::Vec};

pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
pub use sp_runtime::{MultiAddress, Perbill, Permill};

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

use common_primitives::capacity::UnclaimedRewardInfo;
use common_runtime::weights::rocksdb_weights::constants::RocksDbWeight;
pub use common_runtime::{
	constants::MaxSchemaGrants,
	weights,
	weights::{block_weights::BlockExecutionWeight, extrinsic_weights::ExtrinsicBaseWeight},
};
use frame_support::traits::Contains;
#[cfg(feature = "try-runtime")]
use frame_support::traits::{TryStateSelect, UpgradeCheckSelect};
#[allow(deprecated)]
use sp_runtime::traits::transaction_extension::AsTransactionExtension;

mod ethereum;
mod genesis;

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

/// Basefilter to only allow calls to specified transactions to be executed
pub struct BaseCallFilter;

impl Contains<RuntimeCall> for BaseCallFilter {
	fn contains(call: &RuntimeCall) -> bool {
		#[cfg(not(feature = "frequency"))]
		{
			match call {
				RuntimeCall::Utility(pallet_utility_call) =>
					Self::is_utility_call_allowed(pallet_utility_call),
				_ => true,
			}
		}
		#[cfg(feature = "frequency")]
		{
			match call {
				RuntimeCall::Utility(pallet_utility_call) =>
					Self::is_utility_call_allowed(pallet_utility_call),
				// Create provider and create schema are not allowed in mainnet for now. See propose functions.
				RuntimeCall::Msa(pallet_msa::Call::create_provider { .. }) => false,
				RuntimeCall::Schemas(pallet_schemas::Call::create_schema_v3 { .. }) => false,
				// Everything else is allowed on Mainnet
				_ => true,
			}
		}
	}
}

impl BaseCallFilter {
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

			// Block `create_provider` and `create_schema` calls from utility batch
			RuntimeCall::Msa(pallet_msa::Call::create_provider { .. }) |
			RuntimeCall::Schemas(pallet_schemas::Call::create_schema_v3 { .. }) => false,

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
#[allow(deprecated)]
pub type TxExtension = cumulus_pallet_weight_reclaim::StorageWeightReclaim<
	Runtime,
	(
		frame_system::CheckNonZeroSender<Runtime>,
		// merging these types so that we can have more than 12 extensions
		(frame_system::CheckSpecVersion<Runtime>, frame_system::CheckTxVersion<Runtime>),
		frame_system::CheckGenesis<Runtime>,
		frame_system::CheckEra<Runtime>,
		AsTransactionExtension<common_runtime::extensions::check_nonce::CheckNonce<Runtime>>,
		AsTransactionExtension<pallet_frequency_tx_payment::ChargeFrqTransactionPayment<Runtime>>,
		AsTransactionExtension<pallet_msa::CheckFreeExtrinsicUse<Runtime>>,
		AsTransactionExtension<
			pallet_handles::handles_signed_extension::HandlesSignedExtension<Runtime>,
		>,
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

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	fp_self_contained::UncheckedExtrinsic<Address, RuntimeCall, Signature, TxExtension>;

/// Executive: handles dispatch to the various modules.
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

#[derive(Clone)]
pub struct TransactionConverter<B>(PhantomData<B>);

impl<B> Default for TransactionConverter<B> {
	fn default() -> Self {
		Self(PhantomData)
	}
}

impl<B: BlockT> fp_rpc::ConvertTransaction<<B as BlockT>::Extrinsic> for TransactionConverter<B> {
	fn convert_transaction(
		&self,
		transaction: pallet_ethereum::Transaction,
	) -> <B as BlockT>::Extrinsic {
		let extrinsic = UncheckedExtrinsic::new_bare(
			pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
		);
		let encoded = extrinsic.encode();
		<B as BlockT>::Extrinsic::decode(&mut &encoded[..])
			.expect("Encoded extrinsic is always valid")
	}
}

impl fp_self_contained::SelfContainedCall for RuntimeCall {
	type SignedInfo = H160;

	fn is_self_contained(&self) -> bool {
		match self {
			RuntimeCall::Ethereum(call) => call.is_self_contained(),
			_ => false,
		}
	}

	fn check_self_contained(&self) -> Option<Result<Self::SignedInfo, TransactionValidityError>> {
		match self {
			RuntimeCall::Ethereum(call) => call.check_self_contained(),
			_ => None,
		}
	}

	fn validate_self_contained(
		&self,
		info: &Self::SignedInfo,
		dispatch_info: &DispatchInfoOf<RuntimeCall>,
		len: usize,
	) -> Option<TransactionValidity> {
		match self {
			RuntimeCall::Ethereum(call) => call.validate_self_contained(info, dispatch_info, len),
			_ => None,
		}
	}

	fn pre_dispatch_self_contained(
		&self,
		info: &Self::SignedInfo,
		dispatch_info: &DispatchInfoOf<RuntimeCall>,
		len: usize,
	) -> Option<Result<(), TransactionValidityError>> {
		match self {
			RuntimeCall::Ethereum(call) =>
				call.pre_dispatch_self_contained(info, dispatch_info, len),
			_ => None,
		}
	}

	fn apply_self_contained(
		self,
		info: Self::SignedInfo,
	) -> Option<sp_runtime::DispatchResultWithInfo<PostDispatchInfoOf<Self>>> {
		match self {
			call @ RuntimeCall::Ethereum(pallet_ethereum::Call::transact { .. }) =>
				Some(call.dispatch(RuntimeOrigin::from(
					pallet_ethereum::RawOrigin::EthereumTransaction(info),
				))),
			_ => None,
		}
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
	spec_version: 169,
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
	spec_version: 169,
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
	#[cfg(any(not(feature = "frequency-no-relay"), feature = "frequency-lint-check"))]
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
	// The type that provides `Handle` related info for a given `MesssageSourceAccount`
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
	// The mininum size (in bytes) for a schema model
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
			RuntimeCall::Msa(MsaCall::create_sponsored_account_with_delegation {  add_provider_payload, .. }) => Some(capacity_stable_weights::SubstrateWeight::<Runtime>::create_sponsored_account_with_delegation(add_provider_payload.schema_ids.len() as u32)),
			RuntimeCall::Msa(MsaCall::grant_delegation { add_provider_payload, .. }) => Some(capacity_stable_weights::SubstrateWeight::<Runtime>::grant_delegation(add_provider_payload.schema_ids.len() as u32)),
			&RuntimeCall::Msa(MsaCall::add_recovery_commitment { .. }) => Some(
				capacity_stable_weights::SubstrateWeight::<Runtime>::add_recovery_commitment()
			),
			RuntimeCall::Messages(MessagesCall::add_ipfs_message { .. }) => Some(capacity_stable_weights::SubstrateWeight::<Runtime>::add_ipfs_message()),
			RuntimeCall::Messages(MessagesCall::add_onchain_message { payload, .. }) => Some(capacity_stable_weights::SubstrateWeight::<Runtime>::add_onchain_message(payload.len() as u32)),
			RuntimeCall::StatefulStorage(StatefulStorageCall::apply_item_actions { actions, ..}) => Some(capacity_stable_weights::SubstrateWeight::<Runtime>::apply_item_actions(StatefulStorage::sum_add_actions_bytes(actions))),
			RuntimeCall::StatefulStorage(StatefulStorageCall::upsert_page { payload, ..}) => Some(capacity_stable_weights::SubstrateWeight::<Runtime>::upsert_page(payload.len() as u32)),
			RuntimeCall::StatefulStorage(StatefulStorageCall::delete_page { .. }) => Some(capacity_stable_weights::SubstrateWeight::<Runtime>::delete_page()),
			RuntimeCall::StatefulStorage(StatefulStorageCall::apply_item_actions_with_signature_v2 { payload, ..}) => Some(capacity_stable_weights::SubstrateWeight::<Runtime>::apply_item_actions_with_signature(StatefulStorage::sum_add_actions_bytes(&payload.actions))),
            RuntimeCall::StatefulStorage(StatefulStorageCall::upsert_page_with_signature_v2 { payload, ..}) => Some(capacity_stable_weights::SubstrateWeight::<Runtime>::upsert_page_with_signature(payload.payload.len() as u32 )),
            RuntimeCall::StatefulStorage(StatefulStorageCall::delete_page_with_signature_v2 { .. }) => Some(capacity_stable_weights::SubstrateWeight::<Runtime>::delete_page_with_signature()),			RuntimeCall::Handles(HandlesCall::claim_handle { payload, .. }) => Some(capacity_stable_weights::SubstrateWeight::<Runtime>::claim_handle(payload.base_handle.len() as u32)),
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
#[cfg(any(not(feature = "frequency-no-relay"), feature = "frequency-lint-check"))]
impl cumulus_pallet_parachain_system::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnSystemEvent = ();
	type SelfParaId = parachain_info::Pallet<Runtime>;
	type DmpQueue = frame_support::traits::EnqueueWithOrigin<(), sp_core::ConstU8<0>>;
	type ReservedDmpWeight = ();
	type OutboundXcmpMessageSource = ();
	type XcmpMessageHandler = ();
	type ReservedXcmpWeight = ();
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
// ------------------------- Frontier pallets -------------------------//
// pub struct FindAuthorTruncated<F>(PhantomData<F>);
// impl<F: FindAuthor<u32>> FindAuthor<H160> for FindAuthorTruncated<F> {
// 	fn find_author<'a, I>(digests: I) -> Option<H160>
// 	where
// 		I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
// 	{
// 		if let Some(author_index) = F::find_author(digests) {
// 			let authority_id =
// 				pallet_aura::Authorities::<Runtime>::get()[author_index as usize].clone();
// 			return Some(H160::from_slice(&authority_id.to_raw_vec()[4..24]));
// 		}
// 		None
// 	}
// }

pub struct EthAddressMapping;

impl AddressMapping<AccountId> for EthAddressMapping {
	fn into_account_id(address: H160) -> AccountId {
		EthereumAddressMapper::to_account_id(&address.encode())
	}
}

const BLOCK_GAS_LIMIT: u64 = 75_000_000;
const MAX_POV_SIZE: u64 = 5 * 1024 * 1024;
/// The maximum storage growth per block in bytes.
const MAX_STORAGE_GROWTH: u64 = 400 * 1024;

parameter_types! {
	pub BlockGasLimit: U256 = U256::from(BLOCK_GAS_LIMIT);
	pub const GasLimitPovSizeRatio: u64 = BLOCK_GAS_LIMIT.saturating_div(MAX_POV_SIZE);
	pub const GasLimitStorageGrowthRatio: u64 = BLOCK_GAS_LIMIT.saturating_div(MAX_STORAGE_GROWTH);
	// pub PrecompilesValue: FrontierPrecompiles<Runtime> = FrontierPrecompiles::<_>::new();
	pub WeightPerGas: Weight = Weight::from_parts(weight_per_gas(BLOCK_GAS_LIMIT, NORMAL_DISPATCH_RATIO, MILLISECS_PER_BLOCK), 0);
}

impl pallet_evm::Config for Runtime {
	type AccountProvider = pallet_evm::FrameSystemAccountProvider<Self>;
	// type FeeCalculator = BaseFee;
	type FeeCalculator = ();
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
	// type CallOrigin = EnsureAccountId20;
	type CallOrigin = EnsureAddressTruncated;
	// type WithdrawOrigin = EnsureAccountId20;
	type WithdrawOrigin = EnsureAddressTruncated;
	type AddressMapping = EthAddressMapping;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	// type PrecompilesType = FrontierPrecompiles<Self>;
	type PrecompilesType = ();
	// type PrecompilesValue = PrecompilesValue;
	type PrecompilesValue = ();
	// type ChainId = EVMChainId;
	type ChainId = ConstU64<{ CHAIN_ID as u64 }>;
	type BlockGasLimit = BlockGasLimit;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type OnChargeTransaction = ();
	type OnCreate = ();
	// type FindAuthor = FindAuthorTruncated<Aura>;
	type FindAuthor = ();
	type GasLimitPovSizeRatio = GasLimitPovSizeRatio;
	type GasLimitStorageGrowthRatio = GasLimitStorageGrowthRatio;
	type Timestamp = Timestamp;
	type CreateOriginFilter = ();
	type CreateInnerOriginFilter = ();
	type WeightInfo = pallet_evm::weights::SubstrateWeight<Self>;
}

parameter_types! {
	pub DefaultBaseFeePerGas: U256 = U256::from(1_000_000_000);
	pub DefaultElasticity: Permill = Permill::from_parts(125_000);
}
pub struct BaseFeeThreshold;
impl pallet_base_fee::BaseFeeThreshold for BaseFeeThreshold {
	fn lower() -> Permill {
		Permill::zero()
	}
	fn ideal() -> Permill {
		Permill::from_parts(500_000)
	}
	fn upper() -> Permill {
		Permill::from_parts(1_000_000)
	}
}
impl pallet_base_fee::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Threshold = BaseFeeThreshold;
	type DefaultBaseFeePerGas = DefaultBaseFeePerGas;
	type DefaultElasticity = DefaultElasticity;
}

parameter_types! {
	pub const PostBlockAndTxnHashes: PostLogContent = PostLogContent::BlockAndTxnHashes;
}

impl pallet_ethereum::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type StateRoot = pallet_ethereum::IntermediateStateRoot<Self::Version>;
	type PostLogContent = PostBlockAndTxnHashes;
	type ExtraDataLength = ConstU32<30>;
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
	pub enum Runtime {
		// System support stuff.
		System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>} = 0,
		#[cfg(any(not(feature = "frequency-no-relay"), feature = "frequency-lint-check"))]
		ParachainSystem: cumulus_pallet_parachain_system::{
			Pallet, Call, Config<T>, Storage, Inherent, Event<T> } = 1,
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

		// FRQC Update
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

		// Frontier
		EVM: pallet_evm::{Pallet, Call, Storage, Event<T>} = 68,
		Ethereum: pallet_ethereum::{Pallet, Call, Storage, Event, Origin, Config<T>} = 69,
		BaseFee: pallet_base_fee::{Pallet, Call, Storage, Event} = 70,
	}
);

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	define_benchmarks!(
		// Substrate
		[frame_system, SystemBench::<Runtime>]
		[frame_system_extensions, SystemExtensionsBench::<Runtime>]
		[cumulus_pallet_weight_reclaim, WeightReclaim]
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
		[pallet_evm, EVM],
		[pallet_evm, Ethereum],
	);
}

#[cfg(any(not(feature = "frequency-no-relay"), feature = "frequency-lint-check"))]
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
			]);


			#[cfg(feature = "frequency-testnet")]
			presets.push(sp_genesis_builder::PresetId::from("frequency-testnet"));

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
			let dispatch_weight = match &uxt.0.function {
				RuntimeCall::FrequencyTxPayment(pallet_frequency_tx_payment::Call::pay_with_capacity { .. }) |
				RuntimeCall::FrequencyTxPayment(pallet_frequency_tx_payment::Call::pay_with_capacity_batch_all { .. }) => {
					<<Block as BlockT>::Extrinsic as GetDispatchInfo>::get_dispatch_info(&uxt).call_weight
				},
				_ => {
					Weight::zero()
				}
			};
			FrequencyTxPayment::compute_capacity_fee_details(&uxt.0.function, &dispatch_weight, len)
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

	impl pallet_schemas_runtime_api::SchemasRuntimeApi<Block> for Runtime {
		fn get_by_schema_id(schema_id: SchemaId) -> Option<SchemaResponse> {
			Schemas::get_schema_by_id(schema_id)
		}

		fn get_schema_versions_by_name(schema_name: Vec<u8>) -> Option<Vec<SchemaVersionResponse>> {
			Schemas::get_schema_versions(schema_name)
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

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();
			(list, storage_info)
		}

		#[allow(deprecated, non_local_definitions)]
		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{BenchmarkBatch};

			use frame_system_benchmarking::Pallet as SystemBench;
			impl frame_system_benchmarking::Config for Runtime {}

			use frame_system_benchmarking::extensions::Pallet as SystemExtensionsBench;

			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;
			impl cumulus_pallet_session_benchmarking::Config for Runtime {}

			use frame_support::traits::{WhitelistedStorageKeys, TrackedStorageKey};
			let whitelist: Vec<TrackedStorageKey> = AllPalletsWithSystem::whitelisted_storage_keys();

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);
			add_benchmarks!(params, batches);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}

	impl fp_rpc::EthereumRuntimeRPCApi<Block> for Runtime {
		fn chain_id() -> u64 {
			CHAIN_ID as u64
		}

		fn account_basic(address: H160) -> EVMAccount {
			let (account, _) = pallet_evm::Pallet::<Runtime>::account_basic(&address);
			account
		}

		fn gas_price() -> U256 {
			let (gas_price, _) = <Runtime as pallet_evm::Config>::FeeCalculator::min_gas_price();
			gas_price
		}

		fn account_code_at(address: H160) -> Vec<u8> {
			pallet_evm::AccountCodes::<Runtime>::get(address)
		}

		fn author() -> H160 {
			<pallet_evm::Pallet<Runtime>>::find_author()
		}

		fn storage_at(address: H160, index: U256) -> H256 {
			pallet_evm::AccountStorages::<Runtime>::get(address, H256::from(index.to_big_endian()))
		}

		fn call(
			from: H160,
			to: H160,
			data: Vec<u8>,
			value: U256,
			gas_limit: U256,
			max_fee_per_gas: Option<U256>,
			max_priority_fee_per_gas: Option<U256>,
			nonce: Option<U256>,
			estimate: bool,
			access_list: Option<Vec<(H160, Vec<H256>)>>,
		) -> Result<pallet_evm::CallInfo, sp_runtime::DispatchError> {
			use pallet_evm::GasWeightMapping as _;

			let config = if estimate {
				let mut config = <Runtime as pallet_evm::Config>::config().clone();
				config.estimate = true;
				Some(config)
			} else {
				None
			};

			// Estimated encoded transaction size must be based on the heaviest transaction
			// type (EIP1559Transaction) to be compatible with all transaction types.
			let mut estimated_transaction_len = data.len() +
				// pallet ethereum index: 1
				// transact call index: 1
				// Transaction enum variant: 1
				// chain_id 8 bytes
				// nonce: 32
				// max_priority_fee_per_gas: 32
				// max_fee_per_gas: 32
				// gas_limit: 32
				// action: 21 (enum varianrt + call address)
				// value: 32
				// access_list: 1 (empty vec size)
				// 65 bytes signature
				258;

			if access_list.is_some() {
				estimated_transaction_len += access_list.encoded_size();
			}


			let gas_limit = if gas_limit > U256::from(u64::MAX) {
				u64::MAX
			} else {
				gas_limit.low_u64()
			};
			let without_base_extrinsic_weight = true;

			let (weight_limit, proof_size_base_cost) =
				match <Runtime as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
					gas_limit,
					without_base_extrinsic_weight
				) {
					weight_limit if weight_limit.proof_size() > 0 => {
						(Some(weight_limit), Some(estimated_transaction_len as u64))
					}
					_ => (None, None),
				};

			<Runtime as pallet_evm::Config>::Runner::call(
				from,
				to,
				data,
				value,
				gas_limit.unique_saturated_into(),
				max_fee_per_gas,
				max_priority_fee_per_gas,
				nonce,
				access_list.unwrap_or_default(),
				false,
				true,
				weight_limit,
				proof_size_base_cost,
				config.as_ref().unwrap_or(<Runtime as pallet_evm::Config>::config()),
			).map_err(|err| err.error.into())
		}

		fn create(
			from: H160,
			data: Vec<u8>,
			value: U256,
			gas_limit: U256,
			max_fee_per_gas: Option<U256>,
			max_priority_fee_per_gas: Option<U256>,
			nonce: Option<U256>,
			estimate: bool,
			access_list: Option<Vec<(H160, Vec<H256>)>>,
		) -> Result<pallet_evm::CreateInfo, sp_runtime::DispatchError> {
			use pallet_evm::GasWeightMapping as _;

			let config = if estimate {
				let mut config = <Runtime as pallet_evm::Config>::config().clone();
				config.estimate = true;
				Some(config)
			} else {
				None
			};


			let mut estimated_transaction_len = data.len() +
				// from: 20
				// value: 32
				// gas_limit: 32
				// nonce: 32
				// 1 byte transaction action variant
				// chain id 8 bytes
				// 65 bytes signature
				190;

			if max_fee_per_gas.is_some() {
				estimated_transaction_len += 32;
			}
			if max_priority_fee_per_gas.is_some() {
				estimated_transaction_len += 32;
			}
			if access_list.is_some() {
				estimated_transaction_len += access_list.encoded_size();
			}


			let gas_limit = if gas_limit > U256::from(u64::MAX) {
				u64::MAX
			} else {
				gas_limit.low_u64()
			};
			let without_base_extrinsic_weight = true;

			let (weight_limit, proof_size_base_cost) =
				match <Runtime as pallet_evm::Config>::GasWeightMapping::gas_to_weight(
					gas_limit,
					without_base_extrinsic_weight
				) {
					weight_limit if weight_limit.proof_size() > 0 => {
						(Some(weight_limit), Some(estimated_transaction_len as u64))
					}
					_ => (None, None),
				};

			<Runtime as pallet_evm::Config>::Runner::create(
				from,
				data,
				value,
				gas_limit.unique_saturated_into(),
				max_fee_per_gas,
				max_priority_fee_per_gas,
				nonce,
				access_list.unwrap_or_default(),
				false,
				true,
				weight_limit,
				proof_size_base_cost,
				config.as_ref().unwrap_or(<Runtime as pallet_evm::Config>::config()),
			).map_err(|err| err.error.into())
		}

		fn current_transaction_statuses() -> Option<Vec<TransactionStatus>> {
			pallet_ethereum::CurrentTransactionStatuses::<Runtime>::get()
		}

		fn current_block() -> Option<pallet_ethereum::Block> {
			pallet_ethereum::CurrentBlock::<Runtime>::get()
		}

		fn current_receipts() -> Option<Vec<pallet_ethereum::Receipt>> {
			pallet_ethereum::CurrentReceipts::<Runtime>::get()
		}

		fn current_all() -> (
			Option<pallet_ethereum::Block>,
			Option<Vec<pallet_ethereum::Receipt>>,
			Option<Vec<TransactionStatus>>
		) {
			(
				pallet_ethereum::CurrentBlock::<Runtime>::get(),
				pallet_ethereum::CurrentReceipts::<Runtime>::get(),
				pallet_ethereum::CurrentTransactionStatuses::<Runtime>::get()
			)
		}

		fn extrinsic_filter(
			xts: Vec<<Block as BlockT>::Extrinsic>,
		) -> Vec<EthereumTransaction> {
			xts.into_iter().filter_map(|xt| match xt.0.function {
				RuntimeCall::Ethereum(transact { transaction }) => Some(transaction),
				_ => None
			}).collect::<Vec<EthereumTransaction>>()
		}

		fn elasticity() -> Option<Permill> {
			Some(pallet_base_fee::Elasticity::<Runtime>::get())
		}

		fn gas_limit_multiplier_support() {}

		fn pending_block(
			xts: Vec<<Block as BlockT>::Extrinsic>,
		) -> (Option<pallet_ethereum::Block>, Option<Vec<TransactionStatus>>) {
			for ext in xts.into_iter() {
				let _ = Executive::apply_extrinsic(ext);
			}

			Ethereum::on_finalize(System::block_number() + 1);

			(
				pallet_ethereum::CurrentBlock::<Runtime>::get(),
				pallet_ethereum::CurrentTransactionStatuses::<Runtime>::get()
			)
		}

		fn initialize_pending_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header);
		}
	}

	impl fp_rpc::ConvertTransactionRuntimeApi<Block> for Runtime {
		fn convert_transaction(transaction: EthereumTransaction) -> <Block as BlockT>::Extrinsic {
			UncheckedExtrinsic::new_bare(
				pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
			)
		}
	}

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

	#[test]
	fn runtime_apis_are_populated() {
		assert!(RUNTIME_API_VERSIONS.len() > 0);
	}
}
