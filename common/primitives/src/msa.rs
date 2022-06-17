use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::dispatch::DispatchResult;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::DispatchError;

/// Type alias for a message source identifier.
pub type MessageSourceId = u64;

/// A wrapper to distinguish a MessageSourceId that belongs to a Provider vs a Delegator.
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

/// A type that defines an MSA key and its active state.
#[derive(TypeInfo, Debug, Clone, Decode, Encode, PartialEq, Default, MaxEncodedLen)]
pub struct KeyInfo<BlockNumber> {
	/// Message Source Account identifier.
	pub msa_id: MessageSourceId,
	/// A cryptographic nonce.
	pub nonce: u32,
	/// Block number in which the key is revoked. Block number with value zero represents an active status.
	pub expired: BlockNumber,
}

impl<BlockNumber: Clone> KeyInfo<BlockNumber> {
	/// Serializing KeyInfo type for RPC response.
	pub fn map_to_response<AccountId: Clone>(
		&self,
		key: AccountId,
	) -> KeyInfoResponse<AccountId, BlockNumber> {
		KeyInfoResponse {
			key: key.clone(),
			msa_id: self.msa_id,
			nonce: self.nonce,
			expired: self.expired.clone(),
		}
	}
}

/// A type definition for Provider datum.
#[derive(TypeInfo, Debug, Clone, Decode, Encode, PartialEq, Default, MaxEncodedLen)]
pub struct ProviderInfo<BlockNumber> {
	/// Permissions settings for provider.
	pub permission: u8,
	/// Block number indicating when provider delegation was revoked. Block number of zero indicates an active status.
	pub expired: BlockNumber,
}

/// A wrapper to distiguish a MessageSourceId that belongs to a Provider vs a Delegator.
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
	/// Get MSA indentifier for key.
	fn get_msa_id(key: &Self::AccountId) -> Option<MessageSourceId>;
	/// Get delagation status between provider and delegator.
	fn get_provider_info_of(
		provider: Provider,
		delegator: Delegator,
	) -> Option<ProviderInfo<Self::BlockNumber>>;
	/// Check that a key is associated to an MSA and returns key information.
	fn ensure_valid_msa_key(
		key: &Self::AccountId,
	) -> Result<KeyInfo<Self::BlockNumber>, DispatchError>;

	/// Checks that a delegation relationship exists and is not expird between provider and delegator.
	fn ensure_valid_delegation(provider: Provider, delegator: Delegator) -> DispatchResult;
}

/// A type definition for RPC responding with information about an MSA key.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(TypeInfo, Debug, Clone, Decode, Encode, PartialEq, Default, MaxEncodedLen)]
pub struct KeyInfoResponse<AccountId, BlockNumber> {
	/// Msa account key.
	pub key: AccountId,
	/// The Message source account identifier associated with key.
	pub msa_id: MessageSourceId,
	/// A cryptographic nonce count.
	pub nonce: u32,
	/// Block number in which the key is revoked. Block number with value zero represents an active status.
	pub expired: BlockNumber,
}
