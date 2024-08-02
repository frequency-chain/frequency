use frame_support::{
	dispatch::GetDispatchInfo,
	genesis_builder_helper::{build_config, create_default_config},
	weights::Weight,
};
use scale_info::prelude::vec;
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
	traits::Block as BlockT,
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, DispatchError,
};
use sp_std::prelude::Vec;
use sp_version::RuntimeVersion;

use super::{
	AccountId, Balance, Block, Executive, InherentDataExt, Runtime, RuntimeCall,
	RuntimeGenesisConfig, SessionKeys, System, TransactionPayment, VERSION,
};
use crate::{Capacity, FrequencyTxPayment, Handles, Messages, Msa, Schemas, StatefulStorage};

use common_primitives::{
	capacity::*,
	handles::{BaseHandle, DisplayHandle, HandleResponse, PresumptiveSuffixesResponse},
	messages::MessageResponse,
	msa::{
		DelegationResponse, DelegationValidator, DelegatorId, MessageSourceId, ProviderId,
		SchemaGrant, SchemaGrantValidator,
	},
	node::{BlockNumber, Index},
	rpc::RpcEvent,
	schema::{PayloadLocation, SchemaId, SchemaResponse, SchemaVersionResponse},
	stateful_storage::{ItemizedStoragePageResponse, PaginatedStorageResponse},
};

#[cfg(any(not(feature = "frequency-no-relay"), feature = "frequency-lint-check"))]
use super::{ConsensusHook, ParachainSystem};

#[cfg(feature = "try-runtime")]
use frame_support::traits::{TryStateSelect, UpgradeCheckSelect};

#[cfg(feature = "try-runtime")]
use crate::RuntimeBlockWeights;

pub use common_runtime::constants::SLOT_DURATION;

impl_runtime_apis! {
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
		fn create_default_config() -> Vec<u8> {
			create_default_config::<RuntimeGenesisConfig>()
		}

		fn build_config(config: Vec<u8>) -> sp_genesis_builder::Result {
			build_config::<RuntimeGenesisConfig>(config)
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
					<<Block as BlockT>::Extrinsic as GetDispatchInfo>::get_dispatch_info(&uxt).weight
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
			System::read_events_no_consensus().into_iter().map(|e| (*e).into()).collect()
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
			match Msa::get_granted_schemas_by_msa_id(delegator, None) {
				Ok(x) => x,
				Err(_) => vec![],
			}
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
	}

	impl pallet_capacity_runtime_api::CapacityRuntimeApi<Block, AccountId, Balance, BlockNumber> for Runtime {
		fn list_unclaimed_rewards(who: AccountId) -> Vec<UnclaimedRewardInfo<Balance, BlockNumber>> {
			match Capacity::list_unclaimed_rewards(&who) {
				Ok(rewards) => return rewards.into_inner(),
				Err(_) => return Vec::new(),
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
			use frame_benchmarking::{Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;
			use frame_system_benchmarking::Pallet as SystemBench;
			use cumulus_pallet_session_benchmarking::Pallet as SessionBench;
			use super::*;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();
			return (list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch};
			use super::*;

			use frame_system_benchmarking::Pallet as SystemBench;
			impl frame_system_benchmarking::Config for Runtime {}

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
}
