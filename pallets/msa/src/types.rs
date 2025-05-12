//! Types for the MSA Pallet
#![cfg_attr(not(feature = "std"), no_std)]

use super::*;
use parity_scale_codec::{Decode, Encode};

use core::fmt::Debug;

pub use common_primitives::msa::{
	Delegation, DelegatorId, KeyInfoResponse, MessageSourceId, ProviderId,
};
use common_primitives::{node::BlockNumber, schema::SchemaId};

use scale_info::TypeInfo;

/// Dispatch Empty
pub const EMPTY_FUNCTION: fn(MessageSourceId) -> DispatchResult = |_| Ok(());

/// A type definition for the payload for the following operations:
/// -  Adding an MSA key - `pallet_msa::add_public_key_to_msa`
/// -  Authorizing a token withdrawal to an address associated with a public key - `pallet_msa::withdraw_tokens`
#[derive(
	TypeInfo, RuntimeDebugNoBound, Clone, Decode, DecodeWithMemTracking, Encode, PartialEq, Eq,
)]
#[scale_info(skip_type_params(T))]
pub struct AddKeyData<T: Config> {
	/// Message Source Account identifier
	pub msa_id: MessageSourceId,
	/// The block number at which a signed proof of this payload expires.
	pub expiration: BlockNumberFor<T>,
	/// The public key to be added.
	pub new_public_key: T::AccountId,
}

/// Structure that is signed for granting permissions to a Provider
#[derive(TypeInfo, Clone, Debug, Decode, DecodeWithMemTracking, Encode, PartialEq, Eq)]
pub struct AddProvider {
	/// The provider being granted permissions
	pub authorized_msa_id: MessageSourceId,
	/// Schemas for which publishing grants are authorized.
	/// This is private intended for internal use only.
	pub schema_ids: Vec<SchemaId>,
	/// The block number at which the proof for grant_delegation expires.
	pub expiration: BlockNumber,
}

impl AddProvider {
	/// Create new `AddProvider`
	pub fn new(
		authorized_msa_id: MessageSourceId,
		schema_ids: Option<Vec<SchemaId>>,
		expiration: BlockNumber,
	) -> Self {
		let schema_ids = schema_ids.unwrap_or_default();

		Self { authorized_msa_id, schema_ids, expiration }
	}
}

/// The interface for mutating schemas permissions in a delegation relationship.
pub trait PermittedDelegationSchemas<T: Config> {
	/// Attempt to insert a new schema. Dispatches error when the max allowed schemas are exceeded.
	fn try_insert_schema(&mut self, schema_id: SchemaId) -> Result<(), DispatchError>;

	/// Attempt to insert a collection of schemas. Dispatches error when the max allowed schemas are exceeded.
	fn try_insert_schemas(&mut self, schema_ids: Vec<SchemaId>) -> Result<(), DispatchError> {
		for schema_id in schema_ids.into_iter() {
			self.try_insert_schema(schema_id)?;
		}

		Ok(())
	}

	/// Attempt get and mutate a collection of schemas. Dispatches error when a schema cannot be found.
	fn try_get_mut_schemas(
		&mut self,
		schema_ids: Vec<SchemaId>,
		block_number: BlockNumberFor<T>,
	) -> Result<(), DispatchError> {
		for schema_id in schema_ids.into_iter() {
			self.try_get_mut_schema(schema_id, block_number)?;
		}
		Ok(())
	}

	/// Attempt get and mutate a schema. Dispatches error when a schema cannot be found.
	fn try_get_mut_schema(
		&mut self,
		schema_id: SchemaId,
		block_number: BlockNumberFor<T>,
	) -> Result<(), DispatchError>;
}

/// Implementation of SchemaPermission trait on Delegation type.
impl<T: Config> PermittedDelegationSchemas<T>
	for Delegation<SchemaId, BlockNumberFor<T>, T::MaxSchemaGrantsPerDelegation>
{
	/// Attempt to insert a new schema. Dispatches error when the max allowed schemas are exceeded.
	fn try_insert_schema(&mut self, schema_id: SchemaId) -> Result<(), DispatchError> {
		self.schema_permissions
			.try_insert(schema_id, Default::default())
			.map_err(|_| Error::<T>::ExceedsMaxSchemaGrantsPerDelegation)?;
		Ok(())
	}

	/// Attempt get and mutate a schema. Dispatches error when a schema cannot be found.
	fn try_get_mut_schema(
		&mut self,
		schema_id: SchemaId,
		block_number: BlockNumberFor<T>,
	) -> Result<(), DispatchError> {
		let schema = self
			.schema_permissions
			.get_mut(&schema_id)
			.ok_or(Error::<T>::SchemaNotGranted)?;

		*schema = block_number;

		Ok(())
	}
}
