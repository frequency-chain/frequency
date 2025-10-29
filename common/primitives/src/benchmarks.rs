extern crate alloc;
use crate::{
	msa::{DelegatorId, MessageSourceId, ProviderId},
	schema::{IntentId, IntentSetting, ModelType, PayloadLocation, SchemaId},
};
use alloc::vec::Vec;
use frame_support::dispatch::DispatchResult;
use sp_runtime::DispatchError;

/// A trait for helping setup state for running benchmarks.
/// When implementing loosely coupled pallets accessing the dependent
/// pallet state is not possible. Therefore, an alternative solution to
/// setting up the state is necessary to run benchmarks on an extrinsic.
/// Implementing this trait and adding the runtime-benchmarks feature flag
/// makes it possible for the `messages` pallet to access functions that allow
/// one to set up the necessary state for running benchmarks for messages.
pub trait MsaBenchmarkHelper<AccountId> {
	/// Sets the delegation relationship of between Provider and Delegator.
	fn set_delegation_relationship(
		provider: ProviderId,
		delegator: DelegatorId,
		schemas: Vec<SchemaId>,
	) -> DispatchResult;

	/// Sets a public key to an MSA.
	fn add_key(msa_id: MessageSourceId, key: AccountId) -> DispatchResult;

	/// Creates a new MSA and increments the current maximum ID
	fn create_msa(keys: AccountId) -> Result<MessageSourceId, DispatchError>;
}

impl<AccountId> MsaBenchmarkHelper<AccountId> for () {
	/// Sets the delegation relationship of between Provider and Delegator.
	fn set_delegation_relationship(
		_provider: ProviderId,
		_delegator: DelegatorId,
		_schemas: Vec<SchemaId>,
	) -> DispatchResult {
		Ok(())
	}

	/// Sets a public key to an MSA.
	fn add_key(_msa_id: MessageSourceId, _key: AccountId) -> DispatchResult {
		Ok(())
	}

	/// Creates a new MSA and increments the current maximum ID
	fn create_msa(_keys: AccountId) -> Result<MessageSourceId, DispatchError> {
		Ok(MessageSourceId::default())
	}
}

/// A trait for Schema pallet helping setup state for running benchmarks.
pub trait SchemaBenchmarkHelper {
	/// Sets the schema count.
	fn set_schema_count(schema_id: SchemaId);

	/// Sets the Intent count.
	fn set_intent_count(intent_id: IntentId);

	/// Creates a new schema.
	fn create_schema(
		intent_id: IntentId,
		model: Vec<u8>,
		model_type: ModelType,
		payload_location: PayloadLocation,
	) -> Result<SchemaId, DispatchError>;

	/// Creates a new Intent
	fn create_intent(
		name_payload: Vec<u8>,
		payload_location: PayloadLocation,
		settings: Vec<IntentSetting>,
	) -> Result<IntentId, DispatchError>;
}

impl SchemaBenchmarkHelper for () {
	/// Sets the schema count.
	fn set_schema_count(_schema_id: SchemaId) {}

	fn set_intent_count(_intent_id: IntentId) {}

	/// Adds a new schema.
	fn create_schema(
		_intent_id: IntentId,
		_model: Vec<u8>,
		_model_type: ModelType,
		_payload_location: PayloadLocation,
	) -> Result<SchemaId, DispatchError> {
		Ok(SchemaId::default())
	}

	/// Adds a new Intent
	fn create_intent(
		_name_payload: Vec<u8>,
		_payload_location: PayloadLocation,
		_settings: Vec<IntentSetting>,
	) -> Result<IntentId, DispatchError> {
		Ok(IntentId::default())
	}
}

/// A trait for helping setup state for running benchmarks in Capacity.
pub trait RegisterProviderBenchmarkHelper {
	/// Registers a provider
	fn create(target_id: MessageSourceId, name: Vec<u8>) -> DispatchResult;
}

/// A blank implementation
impl RegisterProviderBenchmarkHelper for () {
	fn create(_target_id: MessageSourceId, _name: Vec<u8>) -> DispatchResult {
		Ok(())
	}
}
