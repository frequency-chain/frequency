//! Types for the MSA Pallet
use super::*;
use codec::{Decode, Encode};
pub use common_primitives::{
	msa::{Delegator, KeyInfoResponse, MessageSourceId, Provider},
	schema::SchemaId,
};

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

/// Structure that is signed for granting permissions to a Provider
#[derive(TypeInfo, Clone, Debug, Decode, Encode, PartialEq, Eq)]
pub struct AddProvider<Size: Get<u32>> {
	/// The provider being granted permissions
	pub authorized_msa_id: MessageSourceId,
	/// The permissions granted
	pub permission: u8,
	/// Schemas for which publishing grants are authorized.
	pub granted_schemas: OrderedSet<SchemaId, Size>,
}
