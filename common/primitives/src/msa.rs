use codec::{Decode, Encode, EncodeLike, Error, MaxEncodedLen};
use frame_support::{dispatch::DispatchResult, traits::Get, BoundedVec};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::DispatchError;
use sp_std::prelude::Vec;

pub use crate::{ds::OrderedSetExt, schema::SchemaId};

/// The gap between the current block and a future expiring block allowed when validating signature proofs.
/// 150 blocks at 6 seconds per block would equate to a 15 minute gap.
pub const EXPIRATION_BLOCK_VALIDITY_GAP: u32 = 150;

/// Message Source Id or msaId is the unique identifier for Message Source Accounts
pub type MessageSourceId = u64;

/// A Delegator is a role for an MSA to play.
/// Delegators delegate to Providers.
#[derive(TypeInfo, Debug, Clone, Copy, PartialEq, MaxEncodedLen, Eq)]
pub struct Delegator(pub MessageSourceId);

impl EncodeLike for Delegator {}

impl Encode for Delegator {
	fn encode(&self) -> Vec<u8> {
		self.0.encode()
	}
}

impl Decode for Delegator {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		match <u64>::decode(input) {
			Ok(x) => Ok(Delegator(x)),
			_ => Err(Error::from("Could not decode Delegator")),
		}
	}
}

impl From<MessageSourceId> for Delegator {
	fn from(t: MessageSourceId) -> Self {
		Delegator(t)
	}
}

impl From<Delegator> for MessageSourceId {
	fn from(t: Delegator) -> MessageSourceId {
		t.0
	}
}

/// Struct for the information of the relationship between an MSA and a Provider
#[derive(TypeInfo, Debug, Clone, Decode, Encode, PartialEq, Default, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxSchemaGrants))]
pub struct ProviderInfo<BlockNumber, MaxSchemaGrants>
where
	MaxSchemaGrants: Get<u32>,
{
	/// Block number the grant will be revoked.
	pub expired: BlockNumber,
	/// Schemas that the provider is allowed to use for a delegated message.
	pub schemas: OrderedSetExt<SchemaId, MaxSchemaGrants>,
}

/// Provider is the recipient of a delegation.
/// It is a subset of an MSA
#[derive(TypeInfo, Debug, Clone, Copy, PartialEq, MaxEncodedLen, Eq)]
pub struct Provider(pub MessageSourceId);

impl EncodeLike for Provider {}

impl Encode for Provider {
	fn encode(&self) -> Vec<u8> {
		self.0.encode()
	}
}

impl Decode for Provider {
	fn decode<I: codec::Input>(input: &mut I) -> Result<Self, codec::Error> {
		match <u64>::decode(input) {
			Ok(x) => Ok(Provider(x)),
			_ => Err(Error::from("Could not decode Provider")),
		}
	}
}

/// This is the metadata associated with a provider. As of now it is just a
/// name, but it will likely be expanded in the future
#[derive(MaxEncodedLen, TypeInfo, Debug, Clone, Decode, Encode, PartialEq, Eq)]
#[scale_info(skip_type_params(T))]
pub struct ProviderMetadata<T>
where
	T: Get<u32>,
{
	/// The provider's name
	pub provider_name: BoundedVec<u8, T>,
}

impl From<MessageSourceId> for Provider {
	fn from(t: MessageSourceId) -> Self {
		Provider(t)
	}
}

impl From<Provider> for MessageSourceId {
	fn from(t: Provider) -> MessageSourceId {
		t.0
	}
}

/// This allows other pallets to resolve MSA information.
pub trait AccountProvider {
	/// Type used to associate a key to an MSA.
	type AccountId;
	/// Type for block number.
	type BlockNumber;
	/// Type for maximum number of schemas that can be granted to a provider.
	type MaxSchemaGrants: Get<u32> + Clone + Eq;

	/// Gets the MSA Id associated with this `AccountId` if any
	/// # Arguments
	/// * `key` - The `AccountId` to lookup
	/// # Returns
	/// * `Option<MessageSourceId>`
	fn get_msa_id(key: &Self::AccountId) -> Option<MessageSourceId>;

	/// Gets the relationship information for this delegator, provider pair
	/// # Arguments
	/// * `delegator` - The `MessageSourceId` that delegated to the provider
	/// * `provider` - The `MessageSourceId` that has been delegated to
	/// # Returns
	/// * `Option<ProviderInfo<Self::BlockNumber>>`
	fn get_provider_info_of(
		delegator: Delegator,
		provider: Provider,
	) -> Option<ProviderInfo<Self::BlockNumber, Self::MaxSchemaGrants>>;
	/// Check that a key is associated to an MSA and returns key information.
	/// Returns a `[DispatchError`] if there is no MSA associated with the key
	/// # Arguments
	/// * `key` - The `AccountId` to lookup
	/// # Returns
	/// * `Result<MessageSourceId, DispatchError>`
	fn ensure_valid_msa_key(key: &Self::AccountId) -> Result<MessageSourceId, DispatchError>;

	/// Validates that the delegator and provider have a relationship at this point
	/// # Arguments
	/// * `provider` - The `MessageSourceId` that has been delegated to
	/// * `delegator` - The `MessageSourceId` that delegated to the provider
	/// # Returns
	/// * [DispatchResult](https://paritytech.github.io/substrate/master/frame_support/dispatch/type.DispatchResult.html) The return type of a Dispatchable in frame.
	fn ensure_valid_delegation(
		provider: Provider,
		delegator: Delegator,
		block_number: Option<Self::BlockNumber>,
	) -> Result<ProviderInfo<Self::BlockNumber, Self::MaxSchemaGrants>, DispatchError>;

	/// Validates if the provider is allowed to use the schema
	/// # Arguments
	/// * `provider` - The `MessageSourceId` that has been delegated to
	/// * `delegator` - The `MessageSourceId` that delegated to the provider
	/// * `schema_id` - The `SchemaId` that the provider is trying to use
	/// # Returns
	/// * [DispatchResult](https://paritytech.github.io/substrate/master/frame_support/dispatch/type.DispatchResult.html) The return type of a Dispatchable in frame.
	fn ensure_valid_schema_grant(
		provider: Provider,
		delegator: Delegator,
		schema_id: SchemaId,
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
