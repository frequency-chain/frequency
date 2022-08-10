use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::dispatch::DispatchResult;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::DispatchError;

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
pub struct ProviderInfo<BlockNumber> {
	/// Specifies a permission granted by the delegator to the provider.
	pub permission: u8,
	/// Block number the grant will be revoked.
	pub expired: BlockNumber,
}

/// Provider is the recipient of a delegation.
/// It is a subset of an MSA
#[derive(TypeInfo, Debug, Clone, Copy, Decode, Encode, PartialEq, MaxEncodedLen, Eq)]
pub struct Provider(pub MessageSourceId);

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

	/// Gets the MSA Id associated with this `AccountId` if any
	/// # Arguments
	/// * `key` - The `AccountId` to lookup
	/// # Returns
	/// * `Option<MessageSourceId>`
	fn get_msa_id(key: &Self::AccountId) -> Option<MessageSourceId>;

	/// Gets the relationship information for this delegator, provider pair
	/// # Arguments
	/// * `provider` - The `MessageSourceId` that has been delegated to
	/// * `delegator` - The `MessageSourceId` that delegated to the provider
	/// # Returns
	/// * `Option<ProviderInfo<Self::BlockNumber>>`
	fn get_provider_info_of(
		provider: Provider,
		delegator: Delegator,
	) -> Option<ProviderInfo<Self::BlockNumber>>;
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
