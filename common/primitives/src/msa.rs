use frame_support::{dispatch::DispatchResult, traits::Get, BoundedBTreeMap, BoundedVec};
use parity_scale_codec::{Decode, DecodeWithMemTracking, Encode, EncodeLike, Error, MaxEncodedLen};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, Zero},
	DispatchError, MultiSignature, RuntimeDebug,
};
extern crate alloc;
use alloc::vec::Vec;

pub use crate::schema::{IntentId, SchemaId};

/// Message Source Id or msaId is the unique identifier for Message Source Accounts
pub type MessageSourceId = u64;

/// Ethereum address type alias
pub use sp_core::H160;

/// Response type for getting Ethereum address as a 20-byte array and checksummed hex string
#[derive(TypeInfo, Encode, Decode)]
pub struct AccountId20Response {
	/// Ethereum address as a 20-byte array
	pub account_id: H160,

	/// Ethereum address as a checksummed 42-byte hex string (including 0x prefix)
	pub account_id_checksummed: alloc::string::String,
}

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

impl DecodeWithMemTracking for DelegatorId {}

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
pub struct DelegationResponse<DelegationIdType, BlockNumber> {
	/// SchemaId of schema for which permission is/was granted
	pub provider_id: ProviderId,
	/// The list of schema permissions grants
	pub permissions: Vec<DelegationGrant<DelegationIdType, BlockNumber>>,
}

/// RPC response for getting schema permission grants
#[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
#[derive(TypeInfo, RuntimeDebug, Clone, Decode, Encode, MaxEncodedLen, Eq)]
pub struct DelegationGrant<DelegationIdType, BlockNumber> {
	/// SchemaId of schema for which permission is/was granted
	pub granted_id: DelegationIdType,
	/// Block number the permission was/will be revoked (0 = not revoked)
	pub revoked_at: BlockNumber,
}

impl<DelegationIdType, BlockNumber> PartialEq for DelegationResponse<DelegationIdType, BlockNumber>
where
	DelegationIdType: PartialEq,
	BlockNumber: PartialEq,
{
	fn eq(&self, other: &Self) -> bool {
		self.provider_id == other.provider_id && self.permissions == other.permissions
	}
}

impl<DelegationIdType, BlockNumber> DelegationGrant<DelegationIdType, BlockNumber> {
	/// Create a new SchemaGrant struct
	pub fn new(granted_id: DelegationIdType, revoked_at: BlockNumber) -> Self {
		DelegationGrant { granted_id, revoked_at }
	}
}

impl<DelegatedIdType, BlockNumber> PartialEq for DelegationGrant<DelegatedIdType, BlockNumber>
where
	DelegatedIdType: PartialEq,
	BlockNumber: PartialEq,
{
	fn eq(&self, other: &Self) -> bool {
		self.granted_id == other.granted_id && self.revoked_at == other.revoked_at
	}
}

/// Struct for the information of the relationship between an MSA and a Provider
#[derive(TypeInfo, RuntimeDebug, Clone, Decode, Encode, MaxEncodedLen, Eq)]
#[scale_info(skip_type_params(MaxGrantsPerDelegation))]
pub struct Delegation<DelegatedIdType, BlockNumber, MaxGrantsPerDelegation>
where
	MaxGrantsPerDelegation: Get<u32>,
{
	/// Block number the grant will be revoked.
	pub revoked_at: BlockNumber,
	/// Schemas that the provider is allowed to use for a delegated message.
	pub permissions: BoundedBTreeMap<DelegatedIdType, BlockNumber, MaxGrantsPerDelegation>,
}

// Cannot derive the PartialEq without a mess of impl PartialEq for MaxSchemaGrantsPerDelegation
impl<DelegatedIdType, BlockNumber, MaxGrantsPerDelegation> PartialEq
	for Delegation<DelegatedIdType, BlockNumber, MaxGrantsPerDelegation>
where
	DelegatedIdType: PartialEq,
	BlockNumber: PartialEq,
	MaxGrantsPerDelegation: Get<u32>,
{
	fn eq(&self, other: &Self) -> bool {
		self.revoked_at == other.revoked_at && self.permissions == other.permissions
	}
}

impl<
		DelegatedIdType: Ord + Default,
		BlockNumber: Ord + Copy + Zero + AtLeast32BitUnsigned + Default,
		MaxGrantsPerDelegation: Get<u32>,
	> Default for Delegation<DelegatedIdType, BlockNumber, MaxGrantsPerDelegation>
{
	/// Provides the default values for Delegation type.
	fn default() -> Self {
		Delegation {
			revoked_at: BlockNumber::default(),
			permissions: BoundedBTreeMap::<
				DelegatedIdType,
				BlockNumber,
				MaxGrantsPerDelegation,
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

impl DecodeWithMemTracking for ProviderId {}

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

	/// Gets the max MSA ID from the pallet
	fn get_max_msa_id() -> MessageSourceId;
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
	/// Type for maximum number of items that can be granted to a provider.
	type MaxGrantsPerDelegation: Get<u32>;
	/// Schema Id is the unique identifier for a Schema
	type DelegationId;

	/// Gets the relationship information for this delegator, provider pair
	fn get_delegation_of(
		delegator: DelegatorId,
		provider: ProviderId,
	) -> Option<Delegation<Self::DelegationId, Self::BlockNumber, Self::MaxGrantsPerDelegation>>;
}

/// A behavior that allows for validating a delegator-provider relationship
pub trait DelegationValidator {
	/// Type for block number.
	type BlockNumber;
	/// Type for maximum number of items that can be granted to a provider.
	type MaxGrantsPerDelegation: Get<u32>;
	/// The unique identifier for a delegated item
	type DelegationIdType;

	/// Validates that the delegator and provider have a relationship at this point
	fn ensure_valid_delegation(
		provider: ProviderId,
		delegator: DelegatorId,
		block_number: Option<Self::BlockNumber>,
	) -> Result<
		Delegation<Self::DelegationIdType, Self::BlockNumber, Self::MaxGrantsPerDelegation>,
		DispatchError,
	>;
}

/// A behavior that allows for validating a schema grant
pub trait GrantValidator<DelegationIdType, BlockNumber> {
	// /// The id type of the items being delegated
	// type DelegationIdType;
	// /// The block number type
	// type BlockNumber;
	
	/// Validates if the provider is allowed to use the particular delegated item id currently
	fn ensure_valid_grant(
		provider_id: ProviderId,
		delegator_id: DelegatorId,
		id_to_check: DelegationIdType,
		block_number: BlockNumber,
	) -> DispatchResult;
}

/// A trait that allows checking whether adding a key may be subsidized
pub trait MsaKeyProvider {
	/// the type to use for looking up keys in storage.
	type AccountId;
	/// Returns whether adding `new_key` to `msa_id` may be subsidized
	fn key_eligible_for_subsidized_addition(
		old_key: Self::AccountId,
		new_key: Self::AccountId,
		msa_id: MessageSourceId,
	) -> bool;
}

/// RPC Response for getting MSA keys
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
