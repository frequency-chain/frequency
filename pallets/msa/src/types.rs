//! Types for the MSA Pallet
#![cfg_attr(not(feature = "std"), no_std)]
use super::*;
use codec::{Decode, Encode};

use core::fmt::Debug;

pub use common_primitives::msa::{Delegator, KeyInfoResponse, MessageSourceId, Provider};
use common_primitives::{node::BlockNumber, schema::SchemaId};

use scale_info::TypeInfo;

/// Dispatch Empty
pub const EMPTY_FUNCTION: fn(MessageSourceId) -> DispatchResult = |_| Ok(());

/// A type definition for the payload of adding an MSA key - `pallet_msa::add_public_key_to_msa`
#[derive(TypeInfo, Debug, Clone, Decode, Encode, PartialEq, Eq)]
pub struct AddKeyData {
	/// Message Source Account identifier
	pub msa_id: MessageSourceId,
	/// The block number at which the signed proof for add_public_key_to_msa expires.
	pub expiration: BlockNumber,
}

/// Structure that is signed for granting permissions to a Provider
#[derive(TypeInfo, Clone, Debug, Decode, Encode, PartialEq, Eq)]
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
		let schema_ids = match schema_ids {
			Some(schemas) => schemas,
			None => Vec::default(),
		};

		Self { authorized_msa_id, schema_ids, expiration }
	}
}
