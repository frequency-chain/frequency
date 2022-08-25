//! Types for the MSA Pallet
#![cfg_attr(not(feature = "std"), no_std)]
use super::*;
use codec::{Decode, Encode};
use core::fmt::Debug;

pub use common_primitives::msa::{Delegator, KeyInfoResponse, MessageSourceId, Provider};
use common_primitives::schema::SchemaId;
use orml_utilities::OrderedSet;
use scale_info::TypeInfo;

/// Dispatch Empty
pub const EMPTY_FUNCTION: fn(MessageSourceId) -> DispatchResult = |_| Ok(());

/// A type definition for the payload of adding an MSA key - `pallet_msa::add_key_to_msa`
#[derive(TypeInfo, Debug, Clone, Decode, Encode, PartialEq, Eq)]
pub struct AddKeyData {
	/// Message Source Account identifier
	pub msa_id: MessageSourceId,
	/// A cryptographic nonce.
	pub nonce: u32,
}

/// Extending OrderSet and implement struct OrderedSetExt
#[derive(TypeInfo, Clone, Decode, Encode, PartialEq, Eq)]
#[scale_info(skip_type_params(T, S))]
pub struct OrderedSetExt<
	T: Ord + Encode + Decode + MaxEncodedLen + Clone + Eq + PartialEq,
	S: Get<u32>,
>(OrderedSet<T, S>);

impl<T, S> OrderedSetExt<T, S>
where
	T: Ord + Encode + Decode + MaxEncodedLen + Clone + Eq + PartialEq + core::fmt::Debug,
	S: Get<u32>,
{
	/// Create a new empty set
	pub fn new() -> Self {
		Self(OrderedSet::<T, S>::new())
	}
}

impl<T, S> core::fmt::Debug for OrderedSetExt<T, S>
where
	T: Ord + Encode + Decode + MaxEncodedLen + Clone + Eq + PartialEq + core::fmt::Debug,
	S: Get<u32>,
{
	fn fmt(&self, _f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

/// Structure that is signed for granting permissions to a Provider
#[derive(TypeInfo, Clone, Debug, Decode, Encode, PartialEq, Eq)]
#[scale_info(skip_type_params(MaxDataSize))]
pub struct AddProvider<MaxDataSize>
where
	MaxDataSize: Get<u32> + Clone + Eq,
{
	/// The provider being granted permissions
	pub authorized_msa_id: MessageSourceId,
	/// The permissions granted
	pub permission: u8,
	/// Schemas for which publishing grants are authorized.
	/// This is private intended for internal use only.
	pub granted_schemas: OrderedSetExt<SchemaId, MaxDataSize>,
}

impl<MaxDataSize> AddProvider<MaxDataSize>
where
	MaxDataSize: Get<u32> + Clone + Eq,
{
	/// Create new `AddProvider`
	pub fn new(
		authorized_msa_id: MessageSourceId,
		permission: u8,
		granted_schemas: Option<OrderedSetExt<SchemaId, MaxDataSize>>,
	) -> Self {
		let granted_schemas = match granted_schemas {
			Some(schemas) => schemas,
			None => OrderedSetExt::new(),
		};

		Self { authorized_msa_id, permission, granted_schemas }
	}
}
