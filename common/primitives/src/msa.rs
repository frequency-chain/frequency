use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::dispatch::DispatchResult;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::DispatchError;

pub type MessageSourceId = u64;

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

#[derive(TypeInfo, Debug, Clone, Decode, Encode, PartialEq, Default, MaxEncodedLen)]
pub struct KeyInfo<BlockNumber> {
	pub msa_id: MessageSourceId,
	pub nonce: u32,
	pub expired: BlockNumber,
}

impl<BlockNumber: Clone> KeyInfo<BlockNumber> {
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

#[derive(TypeInfo, Debug, Clone, Decode, Encode, PartialEq, Default, MaxEncodedLen)]
pub struct ProviderInfo<BlockNumber> {
	pub permission: u8,
	pub expired: BlockNumber,
}

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

pub trait AccountProvider {
	type AccountId;
	type BlockNumber;
	fn get_msa_id(key: &Self::AccountId) -> Option<MessageSourceId>;
	fn get_provider_info_of(
		provider: Provider,
		delegator: Delegator,
	) -> Option<ProviderInfo<Self::BlockNumber>>;
	fn ensure_valid_msa_key(
		key: &Self::AccountId,
	) -> Result<KeyInfo<Self::BlockNumber>, DispatchError>;

	fn ensure_valid_delegation(provider: Provider, delegator: Delegator) -> DispatchResult;
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(TypeInfo, Debug, Clone, Decode, Encode, PartialEq, Default, MaxEncodedLen)]
pub struct KeyInfoResponse<AccountId, BlockNumber> {
	pub key: AccountId,
	pub msa_id: MessageSourceId,
	pub nonce: u32,
	pub expired: BlockNumber,
}
