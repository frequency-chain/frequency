use frame_support::dispatch::DispatchResult;
use sp_std::vec::Vec;

use crate::{
	msa::{DelegatorId, MessageSourceId, ProviderId},
	schema::{ModelType, PayloadLocation, SchemaId},
};

/// A trait for helping setup state for running benchmarks.
/// When implementing loosely coupled pallets accessing the dependent
/// pallet state is not possible. Therefore, an alternative solution to
/// setting up the state is necessary to run benchmarks on an extrinsic.
/// Implementing this trait and adding the runtime-benchmarks feature flag
/// makes it possible for the messages pallet to access functions that allow
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

	/// Sets a publickey to an MSA.
	fn add_key(_msa_id: MessageSourceId, _key: AccountId) -> DispatchResult {
		Ok(())
	}
}

/// A trait for Schema pallet helping setup state for running benchmarks.
pub trait SchemaBenchmarkHelper {
	/// Sets the schema count.
	fn set_schema_count(schema_id: SchemaId);

	/// Creates a new schema.
	fn create_schema(
		model: Vec<u8>,
		model_type: ModelType,
		payload_location: PayloadLocation,
	) -> DispatchResult;
}

impl SchemaBenchmarkHelper for () {
	/// Sets the schema count.
	fn set_schema_count(_schema_id: SchemaId) {}

	/// Adds a new schema.
	fn create_schema(
		_model: Vec<u8>,
		_model_type: ModelType,
		_payload_location: PayloadLocation,
	) -> DispatchResult {
		Ok(())
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
