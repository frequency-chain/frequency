use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{dispatch::DispatchResult, traits::Get, BoundedVec};
use orml_utilities::OrderedSet;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::DispatchError;

use crate::schema::SchemaId;

/// Message Source Id or msaId is the unique identifier for Message Source Accounts
pub type MessageSourceId = u64;

/// A Delegator is a role for an MSA to play.
/// Delegators delegate to Providers.
#[derive(TypeInfo, Debug, Clone, Copy, Decode, Encode, PartialEq, MaxEncodedLen, Eq)]
pub struct Delegator(pub MessageSourceId);

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

/// KeyInfo holds the information on the relationship between a key and an MSA
#[derive(TypeInfo, Debug, Clone, Decode, Encode, PartialEq, Default, MaxEncodedLen)]
pub struct KeyInfo {
	/// The Message Source Account that this key is associated with
	pub msa_id: MessageSourceId,
	/// Prevent key addition replays
	pub nonce: u32,
}

impl KeyInfo {
	/// Convert `KeyInfo` into `KeyInfoResponse`
	/// # Arguments
	/// * `key` - The `AccountId` for self
	/// # Returns
	/// * `KeyInfoResponse<AccountId, BlockNumber>`
	pub fn map_to_response<AccountId: Clone>(&self, key: AccountId) -> KeyInfoResponse<AccountId> {
		KeyInfoResponse { key, msa_id: self.msa_id, nonce: self.nonce }
	}
}

/// Struct for the information of the relationship between an MSA and a Provider
#[derive(TypeInfo, Debug, Clone, Decode, Encode, PartialEq, Default, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxSchemaGrants))]
pub struct ProviderInfo<BlockNumber, MaxSchemaGrants>
where
	MaxSchemaGrants: Get<u32> + Clone + Eq,
{
	/// Specifies a permission granted by the delegator to the provider.
	pub permission: u8,
	/// Block number the grant will be revoked.
	pub expired: BlockNumber,
	/// Schemas that the provider is allowed to use for a delegated message.
	pub schemas: OrderedSetExt<SchemaId, MaxSchemaGrants>,
}

/// Provider is the recipient of a delegation.
/// It is a subset of an MSA
#[derive(TypeInfo, Debug, Clone, Copy, Decode, Encode, PartialEq, MaxEncodedLen, Eq)]
pub struct Provider(pub MessageSourceId);

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
	/// * `Result<KeyInfo, DispatchError>`
	fn ensure_valid_msa_key(key: &Self::AccountId) -> Result<KeyInfo, DispatchError>;

	/// Validates that the delegator and provider have a relationship at this point
	/// # Arguments
	/// * `provider` - The `MessageSourceId` that has been delegated to
	/// * `delegator` - The `MessageSourceId` that delegated to the provider
	/// # Returns
	/// * [DispatchResult](https://paritytech.github.io/substrate/master/frame_support/dispatch/type.DispatchResult.html) The return type of a Dispatchable in frame.
	fn ensure_valid_delegation(provider: Provider, delegator: Delegator) -> DispatchResult;
}

/// RPC Response form of [`KeyInfo`]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(TypeInfo, Debug, Clone, Decode, Encode, PartialEq, Default, MaxEncodedLen)]
pub struct KeyInfoResponse<AccountId> {
	/// The `AccountId` associated with the `msa_id`
	pub key: AccountId,
	/// The MSA associated with the `key`
	pub msa_id: MessageSourceId,
	/// The nonce value for signed updates to this data
	pub nonce: u32,
}

/// Extending OrderSet and implement struct OrderedSetExt
#[derive(TypeInfo, Clone, Decode, Encode, PartialEq, Eq, MaxEncodedLen, Default)]
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
