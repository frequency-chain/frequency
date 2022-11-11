use codec::{Decode, Encode, EncodeLike, Error, MaxEncodedLen};
use frame_support::{
	dispatch::DispatchResult, traits::Get, BoundedBTreeMap, BoundedVec, RuntimeDebug,
};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, Zero},
	DispatchError,
};
use sp_std::prelude::Vec;

pub use crate::schema::SchemaId;

/// Message Source Id or msaId is the unique identifier for Message Source Accounts
pub type MessageSourceId = u64;

/// Maximum # of providers per delegator for deletion/benchmarking use
pub const MAX_NUMBER_OF_PROVIDERS_PER_DELEGATOR: u32 = 128;

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
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
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

/// Struct for the information of the relationship between an MSA and a Provider
#[derive(TypeInfo, RuntimeDebug, Clone, Decode, Encode, PartialEq, MaxEncodedLen, Eq)]
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
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
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
	type MaxSchemaGrantsPerDelegation: Get<u32> + Clone + Eq;
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
	type MaxSchemaGrantsPerDelegation: Get<u32> + Clone + Eq;
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

/// RPC Response for getting getting MSA ids
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(TypeInfo, Debug, Clone, Decode, Encode, PartialEq, Default, MaxEncodedLen)]
pub struct KeyInfoResponse<AccountId> {
	/// The `AccountId` associated with the `msa_id`
	pub key: AccountId,
	/// The MSA associated with the `key`
	pub msa_id: MessageSourceId,
}
