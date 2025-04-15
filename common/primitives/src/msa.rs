use frame_support::{dispatch::DispatchResult, traits::Get, BoundedBTreeMap, BoundedVec};
use parity_scale_codec::{Decode, Encode, EncodeLike, Error, MaxEncodedLen};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, Zero},
	DispatchError, MultiSignature, RuntimeDebug,
};
extern crate alloc;
use alloc::vec::Vec;

pub use crate::schema::SchemaId;

/// Message Source Id or msaId is the unique identifier for Message Source Accounts
pub type MessageSourceId = u64;

/// A DelegatorId an MSA Id serving the role of a Delegator.
/// Delegators delegate to Providers.
/// Encodes and Decodes as just a `u64`
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(TypeInfo, Default, Debug, Clone, Copy, PartialEq, MaxEncodedLen, Eq)]
pub struct DelegatorId(pub MessageSourceId);

impl EncodeLike for DelegatorId {}

impl Encode for DelegatorId {
	fn encode(&self) -> Vec<u8> {
		self.0.encode()
	}
}

impl Decode for DelegatorId {
	fn decode<I: parity_scale_codec::Input>(
		input: &mut I,
	) -> Result<Self, parity_scale_codec::Error> {
		match <u64>::decode(input) {
			Ok(x) => Ok(DelegatorId(x)),
			_ => Err(Error::from("Could not decode DelegatorId")),
		}
	}
}

impl From<MessageSourceId> for DelegatorId {
	fn from(t: MessageSourceId) -> Self {
		DelegatorId(t)
	}
}

impl From<DelegatorId> for MessageSourceId {
	fn from(t: DelegatorId) -> MessageSourceId {
		t.0
	}
}

/// RPC response for getting delegated providers with their permissions
#[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
#[derive(TypeInfo, RuntimeDebug, Clone, Decode, Encode, MaxEncodedLen, Eq)]
pub struct DelegationResponse<SchemaId, BlockNumber> {
	/// SchemaId of schema for which permission is/was granted
	pub provider_id: ProviderId,
	/// The list of schema permissions grants
	pub permissions: Vec<SchemaGrant<SchemaId, BlockNumber>>,
}

/// RPC response for getting schema permission grants
#[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
#[derive(TypeInfo, RuntimeDebug, Clone, Decode, Encode, MaxEncodedLen, Eq)]
pub struct SchemaGrant<SchemaId, BlockNumber> {
	/// SchemaId of schema for which permission is/was granted
	pub schema_id: SchemaId,
	/// Block number the permission was/will be revoked (0 = not revoked)
	pub revoked_at: BlockNumber,
}

impl<SchemaId, BlockNumber> PartialEq for DelegationResponse<SchemaId, BlockNumber>
where
	SchemaId: PartialEq,
	BlockNumber: PartialEq,
{
	fn eq(&self, other: &Self) -> bool {
		self.provider_id == other.provider_id && self.permissions == other.permissions
	}
}

impl<SchemaId, BlockNumber> SchemaGrant<SchemaId, BlockNumber> {
	/// Create a new SchemaGrant struct
	pub fn new(schema_id: SchemaId, revoked_at: BlockNumber) -> Self {
		SchemaGrant { schema_id, revoked_at }
	}
}

impl<SchemaId, BlockNumber> PartialEq for SchemaGrant<SchemaId, BlockNumber>
where
	SchemaId: PartialEq,
	BlockNumber: PartialEq,
{
	fn eq(&self, other: &Self) -> bool {
		self.schema_id == other.schema_id && self.revoked_at == other.revoked_at
	}
}

/// Struct for the information of the relationship between an MSA and a Provider
#[derive(TypeInfo, RuntimeDebug, Clone, Decode, Encode, MaxEncodedLen, Eq)]
#[scale_info(skip_type_params(MaxSchemaGrantsPerDelegation))]
pub struct Delegation<SchemaId, BlockNumber, MaxSchemaGrantsPerDelegation>
where
	MaxSchemaGrantsPerDelegation: Get<u32>,
{
	/// Block number the grant will be revoked.
	pub revoked_at: BlockNumber,
	/// Schemas that the provider is allowed to use for a delegated message.
	pub schema_permissions: BoundedBTreeMap<SchemaId, BlockNumber, MaxSchemaGrantsPerDelegation>,
}

// Cannot derive the PartialEq without a mess of impl PartialEq for MaxSchemaGrantsPerDelegation
impl<SchemaId, BlockNumber, MaxSchemaGrantsPerDelegation> PartialEq
	for Delegation<SchemaId, BlockNumber, MaxSchemaGrantsPerDelegation>
where
	SchemaId: PartialEq,
	BlockNumber: PartialEq,
	MaxSchemaGrantsPerDelegation: Get<u32>,
{
	fn eq(&self, other: &Self) -> bool {
		self.revoked_at == other.revoked_at && self.schema_permissions == other.schema_permissions
	}
}

impl<
		SchemaId: Ord + Default,
		BlockNumber: Ord + Copy + Zero + AtLeast32BitUnsigned + Default,
		MaxSchemaGrantsPerDelegation: Get<u32>,
	> Default for Delegation<SchemaId, BlockNumber, MaxSchemaGrantsPerDelegation>
{
	/// Provides the default values for Delegation type.
	fn default() -> Self {
		Delegation {
			revoked_at: BlockNumber::default(),
			schema_permissions: BoundedBTreeMap::<
				SchemaId,
				BlockNumber,
				MaxSchemaGrantsPerDelegation,
			>::new(),
		}
	}
}

/// Provider is the recipient of a delegation.
/// It is a subset of an MSA
/// Encodes and Decodes as just a `u64`
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(TypeInfo, Default, Debug, Clone, Copy, PartialEq, MaxEncodedLen, Eq)]
pub struct ProviderId(pub MessageSourceId);

impl EncodeLike for ProviderId {}

impl Encode for ProviderId {
	fn encode(&self) -> Vec<u8> {
		self.0.encode()
	}
}

impl Decode for ProviderId {
	fn decode<I: parity_scale_codec::Input>(
		input: &mut I,
	) -> Result<Self, parity_scale_codec::Error> {
		match <u64>::decode(input) {
			Ok(x) => Ok(ProviderId(x)),
			_ => Err(Error::from("Could not decode ProviderId")),
		}
	}
}

impl From<MessageSourceId> for ProviderId {
	fn from(t: MessageSourceId) -> Self {
		ProviderId(t)
	}
}

impl From<ProviderId> for MessageSourceId {
	fn from(t: ProviderId) -> MessageSourceId {
		t.0
	}
}

/// This is the metadata associated with a provider. As of now it is just a
/// name, but it will likely be expanded in the future
#[derive(MaxEncodedLen, TypeInfo, Debug, Clone, Decode, Encode, PartialEq, Eq)]
#[scale_info(skip_type_params(T))]
pub struct ProviderRegistryEntry<T>
where
	T: Get<u32>,
{
	/// The provider's name
	pub provider_name: BoundedVec<u8, T>,
}

/// The pointer value for the Signature Registry
#[derive(MaxEncodedLen, TypeInfo, Debug, Clone, Decode, Encode, PartialEq, Eq)]
pub struct SignatureRegistryPointer<BlockNumber> {
	/// The newest signature that will be added to the registry when we get the next newest
	pub newest: MultiSignature,

	/// Block number that `newest` expires at
	pub newest_expires_at: BlockNumber,

	/// Pointer to the oldest signature in the list
	pub oldest: MultiSignature,

	/// Count of signatures in the registry
	/// Will eventually match the `MaxSignaturesStored`, but during initialization is needed to fill the list
	pub count: u32,
}

/// A behavior that allows looking up an MSA id
pub trait MsaLookup {
	/// The association between key and MSA
	type AccountId;

	/// Gets the MSA Id associated with this `AccountId` if any
	fn get_msa_id(key: &Self::AccountId) -> Option<MessageSourceId>;
}

/// A behavior that allows for validating an MSA
pub trait MsaValidator {
	/// The association between key and MSA
	type AccountId;

	/// Check that a key is associated to an MSA and returns key information.
	/// Returns a [`DispatchError`] if there is no MSA associated with the key
	fn ensure_valid_msa_key(key: &Self::AccountId) -> Result<MessageSourceId, DispatchError>;
}

/// A behavior that allows for looking up delegator-provider relationships
pub trait ProviderLookup {
	/// Type for block number.
	type BlockNumber;
	/// Type for maximum number of schemas that can be granted to a provider.
	type MaxSchemaGrantsPerDelegation: Get<u32>;
	/// Schema Id is the unique identifier for a Schema
	type SchemaId;

	/// Gets the relationship information for this delegator, provider pair
	fn get_delegation_of(
		delegator: DelegatorId,
		provider: ProviderId,
	) -> Option<Delegation<Self::SchemaId, Self::BlockNumber, Self::MaxSchemaGrantsPerDelegation>>;
}

/// A behavior that allows for validating a delegator-provider relationship
pub trait DelegationValidator {
	/// Type for block number.
	type BlockNumber;
	/// Type for maximum number of schemas that can be granted to a provider.
	type MaxSchemaGrantsPerDelegation: Get<u32>;
	/// Schema Id is the unique identifier for a Schema
	type SchemaId;

	/// Validates that the delegator and provider have a relationship at this point
	fn ensure_valid_delegation(
		provider: ProviderId,
		delegator: DelegatorId,
		block_number: Option<Self::BlockNumber>,
	) -> Result<
		Delegation<Self::SchemaId, Self::BlockNumber, Self::MaxSchemaGrantsPerDelegation>,
		DispatchError,
	>;
}

/// A behavior that allows for validating a schema grant
pub trait SchemaGrantValidator<BlockNumber> {
	/// Validates if the provider is allowed to use the particular schema id currently
	fn ensure_valid_schema_grant(
		provider_id: ProviderId,
		delegator_id: DelegatorId,
		schema_id: SchemaId,
		block_number: BlockNumber,
	) -> DispatchResult;
}

/// RPC Response for getting getting MSA keys
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(TypeInfo, Debug, Clone, Decode, Encode, PartialEq, Default, MaxEncodedLen)]
pub struct KeyInfoResponse<AccountId> {
	/// The MSA associated with the `key`
	pub msa_id: MessageSourceId,
	/// The list of `AccountId` associated with the `msa_id`
	pub msa_keys: Vec<AccountId>,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn decoding_provider_id_failure() {
		let mut da: &[u8] = b"\xf6\xf5";
		let decoded = DelegatorId::decode(&mut da);
		assert!(decoded.is_err());
	}

	#[test]
	fn decoding_provider_id_success() {
		let val = 16777215_u64.encode();
		let decoded = ProviderId::decode(&mut &val[..]);
		assert_eq!(decoded, Ok(ProviderId(16777215)))
	}

	#[test]
	fn decoding_delegate_id_failure() {
		let mut da: &[u8] = b"\xf6\xf5";
		let decoded = DelegatorId::decode(&mut da);
		assert!(decoded.is_err());
	}

	#[test]
	fn decoding_delegator_id_success() {
		let val = 42_u64.encode();
		let decoded = DelegatorId::decode(&mut &val[..]);
		assert_eq!(decoded, Ok(DelegatorId(42)))
	}
}
